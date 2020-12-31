/*
 * mini-haskell: light-weight Haskell for fun
 * Copyright (C) 2020  Xie Ruifeng
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU Affero General Public License as
 * published by the Free Software Foundation, either version 3 of the
 * License, or (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU Affero General Public License for more details.
 *
 * You should have received a copy of the GNU Affero General Public License
 * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */

//! Persistent input from a [`std::io::Read`].

use std::cell::UnsafeCell;
use std::rc::Rc;

use crate::rc_view::RcView;

const DEFAULT_BUF_SIZE: usize = 4 * 1024;
const MAXIMUM_RETRY: isize = 5;

/// A "raw" input.
/// - segmented, shared, and immutable back buffer
/// - lazy reading from the input
/// - lightweight cloning
/// - NOT thread-safe
pub struct RawInput<I>(Rc<UnsafeCell<InputSegment<I>>>);

impl<I> Clone for RawInput<I> {
    fn clone(&self) -> Self { RawInput(self.0.clone()) }
}

enum InputSegment<I> {
    EndOfFile {
        io_error: Option<std::io::Error>,
    },
    Cons {
        data: RcView<[u8], str>,
        next: RawInput<I>,
    },
    Invalid {
        data: RcView<[u8], [u8]>,
        next: RawInput<I>,
    },
    Delayed {
        remaining: Option<RcView<[u8], [u8]>>,
        input: I,
    },
}

impl<I> Default for InputSegment<I> {
    fn default() -> Self { InputSegment::EndOfFile { io_error: None } }
}

type DelayedContent<I> = (Option<RcView<[u8], [u8]>>, I);

impl<I> InputSegment<I> {
    fn new(input: I) -> Self {
        InputSegment::Delayed {
            remaining: None,
            input,
        }
    }

    fn is_delayed(&self) -> bool {
        matches!(self, Self::Delayed { .. })
    }

    fn take_delayed(&mut self) -> Option<DelayedContent<I>> {
        match self {
            Self::Delayed { .. } => match std::mem::take(self) {
                Self::Delayed { remaining, input } => Some((remaining, input)),
                _ => unreachable!(),
            },
            _ => None,
        }
    }
}

impl<I> RawInput<I> {
    /// Create a new [`RawInput`] from a [`std::io::Read`].
    pub fn new(input: I) -> Self {
        RawInput(Rc::new(UnsafeCell::new(InputSegment::new(input))))
    }

    fn wrap(segment: InputSegment<I>) -> Self {
        RawInput(Rc::new(UnsafeCell::new(segment)))
    }

    /// Dump out the content of this raw input.
    pub fn dump(&self) {
        let node = unsafe { &mut *self.0.get() };
        match node {
            InputSegment::EndOfFile { .. } => println!("- <EOF>"),
            InputSegment::Delayed { .. } => println!("- <lazy> not yet read"),
            InputSegment::Cons { data, next } => {
                println!("- {:?}", data);
                next.dump()
            }
            InputSegment::Invalid { data, next } => {
                println!("- <invalid> {:?}", data);
                next.dump()
            }
        }
    }
}

impl<I: std::io::Read> RawInput<I> {
    fn prepare(&mut self) {
        let node = unsafe { &mut *self.0.get() };
        let delayed = node.take_delayed();
        if delayed.is_none() { return; }
        let (remaining, mut input) = delayed.unwrap();
        let mut buffer = vec![0u8; DEFAULT_BUF_SIZE];
        let mut to_read = &mut *buffer;
        if let Some(xs) = remaining {
            let n = xs.len();
            let (head, rest) = to_read.split_at_mut(n);
            head.copy_from_slice(&xs);
            to_read = rest;
        }
        let mut retry = MAXIMUM_RETRY;
        let tail = loop {
            match input.read(to_read) {
                Ok(0) if to_read.is_empty() => break InputSegment::new(input),
                Ok(0) => break InputSegment::EndOfFile { io_error: None },
                Ok(n) => to_read = &mut to_read[n..],
                Err(e) => match e.kind() {
                    std::io::ErrorKind::Interrupted if retry > 0 => retry -= 1,
                    _ => break InputSegment::EndOfFile { io_error: Some(e) },
                },
            }
        };
        let n = DEFAULT_BUF_SIZE - to_read.len();
        let buffer = Rc::<[u8]>::from(buffer);
        let to_decode = RcView::new(buffer, |b| &b[..n]);
        *node = Self::decode(to_decode, tail)
    }

    fn decode(to_decode: RcView<[u8], [u8]>, tail: InputSegment<I>) -> InputSegment<I> {
        let rest = &*to_decode;
        if rest.is_empty() { return tail; }
        match std::str::from_utf8(rest) {
            Ok(s) => InputSegment::Cons {
                data: unsafe { to_decode.derive(s) },
                next: RawInput::wrap(tail),
            },
            Err(e) => {
                let n = e.valid_up_to();
                let (valid, rest) = rest.split_at(n);
                let tail = match e.error_len() {
                    None if tail.is_delayed() => match tail {
                        InputSegment::Delayed { remaining, input } => {
                            assert!(matches!(remaining, None));
                            InputSegment::Delayed {
                                remaining: Some(unsafe { to_decode.derive(rest) }),
                                input,
                            }
                        }
                        _ => unreachable!("impossible: no remaining input expected here"),
                    },
                    _ => {
                        let k = e.error_len().unwrap_or_else(|| rest.len());
                        let (invalid, rest) = rest.split_at(k);
                        InputSegment::Invalid {
                            data: unsafe { to_decode.derive(invalid) },
                            next: RawInput::wrap(Self::decode(
                                unsafe { to_decode.derive(rest) }, tail)),
                        }
                    }
                };
                if n == 0 { tail } else {
                    let valid = unsafe { std::str::from_utf8_unchecked(valid) };
                    InputSegment::Cons {
                        data: unsafe { to_decode.derive(valid) },
                        next: RawInput::wrap(tail),
                    }
                }
            }
        }
    }
}

/// Input with the ability to read one character once.
/// Keeping such an iterator will prevent releasing the input resource.
pub struct Input<I> {
    input: RawInput<I>,
    index: usize,
}

impl<I> Clone for Input<I> {
    fn clone(&self) -> Self {
        Self { input: self.input.clone(), index: self.index }
    }
}

impl<I> Input<I> {
    /// Create a new [`Input`] from a [`std::io::Read`].
    pub fn new(input: I) -> Self {
        Input { input: RawInput::new(input), index: 0 }
    }
}

impl<I: std::io::Read> Input<I> {
    /// Get the next character, if any.
    pub fn next(
        mut self,
        mut report: impl FnMut(&[u8]),
    ) -> std::result::Result<(char, Self), impl Into<Option<std::io::Error>>> {
        loop {
            self.input.prepare();
            let head = unsafe { &mut *self.input.0.get() };
            match head {
                InputSegment::EndOfFile { io_error } => {
                    break Err(unsafe { RcView::wrap(self.input.0, io_error) });
                }
                InputSegment::Cons { data, next } => {
                    let mut cs = data[self.index..].chars();
                    match cs.next() {
                        Some(c) => {
                            self.index = data.len() - cs.as_str().len();
                            break Ok((c, self));
                        }
                        None => self = Self { input: next.clone(), index: 0 },
                    }
                }
                InputSegment::Invalid { data, .. } => {
                    report(data);
                    let next = match std::mem::take(head) {
                        InputSegment::Invalid { next, .. } => next,
                        _ => unreachable!("Already pattern matched."),
                    };
                    *head = Rc::try_unwrap(next.0).ok().unwrap().into_inner();
                }
                _ => unreachable!("RawInput::prepare shall not return a Delayed."),
            }
        }
    }

    /// Match on the input, succeed if the input matches the given string.
    pub fn r#match(mut self, s: &str, mut report: impl FnMut(&[u8])) -> Option<Self> {
        let mut s = s.as_bytes();
        loop {
            if s.is_empty() { return Some(self); }
            self.input.prepare();
            let head = unsafe { &mut *self.input.0.get() };
            match head {
                InputSegment::EndOfFile { .. } => break None,
                InputSegment::Cons { data, next } => {
                    let cs = data[self.index..].as_bytes();
                    let n = std::cmp::min(s.len(), cs.len());
                    if s[..n] != cs[..n] { break None; }
                    self.index += n;
                    if cs[n..].is_empty() { self = Self { input: next.clone(), index: 0 }; }
                    s = &s[n..];
                }
                InputSegment::Invalid { data, .. } => {
                    report(data);
                    let next = match std::mem::take(head) {
                        InputSegment::Invalid { next, .. } => next,
                        _ => unreachable!("Already pattern matched."),
                    };
                    *head = Rc::try_unwrap(next.0).ok().unwrap().into_inner();
                }
                _ => unreachable!("RawInput::prepare shall not return a Delayed."),
            }
        }
    }

    /// Dump out the content of this input.
    pub fn dump(&self) {
        println!("Input[index = {}]:", self.index);
        self.input.dump();
    }
}
