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

//! lexical scanner for mini-haskell

use crate::buffer;
use crate::utils::*;

/// Source location.
#[derive(Eq, PartialEq, Debug)]
pub struct Location {
    /// line number, starting from 1.
    pub line: usize,
    /// column number, starting from 1.
    pub column: usize,
    /// offset into the source file, starting from 0.
    pub offset: usize,
}

impl Default for Location {
    fn default() -> Self { Location { line: 1, column: 1, offset: 0 } }
}

impl Location {
    /// Create a new location, the same as `Location::default()`.
    pub fn new() -> Self { Self::default() }
}

/// A back buffer for a scanner.
pub trait Buffer {
    /// Peek the next character without consuming it.
    fn peek(&mut self) -> Option<char>;
    /// Peek no more than `n` characters without consuming them.
    fn peek_n(&mut self, n: usize) -> buffer::Iter;
    /// Take the next character and consume it.
    fn next(&mut self) -> Option<char>;
    /// Take no more than `n` characters and consume them.
    fn next_n(&mut self, n: usize) -> buffer::Iter;
}

trait SetAnchor: Buffer {
    fn set_anchor(&mut self, anchor: Option<usize>) -> Option<usize>;
    fn current_index(&mut self) -> usize;
    fn revert(&mut self);
}

/// Buffer tied with an input iterator.
pub struct NormalBuffer<S: Iterator<Item=char>> {
    buffer: buffer::RingBuffer,
    input: S,
}

impl<S: Iterator<Item=char>> NormalBuffer<S> {
    /// Push no more than `n` characters from the input iterator into the back buffer.
    /// Return the number of characters pushed (less than `n` iff EOF).
    pub fn read_n(&mut self, n: usize) {
        if let Some(n) = greater(n, self.buffer.remaining_count()) {
            self.buffer.push_n(n as usize, &mut self.input);
        }
    }
}

impl<S: Iterator<Item=char>> NormalBuffer<S> {
    /// Create a normal buffer from a character stream.
    pub fn new(input: S) -> Self {
        NormalBuffer { buffer: buffer::RingBuffer::new(), input }
    }
}

impl<S: Iterator<Item=char>> Buffer for NormalBuffer<S> {
    fn peek(&mut self) -> Option<char> {
        self.read_n(1);
        self.buffer.peek()
    }

    fn peek_n(&mut self, n: usize) -> buffer::Iter {
        self.read_n(n);
        self.buffer.peek_n(n)
    }

    fn next(&mut self) -> Option<char> {
        self.read_n(1);
        self.buffer.pop()
    }

    fn next_n(&mut self, n: usize) -> buffer::Iter {
        self.read_n(n);
        self.buffer.pop_n(n)
    }
}

impl<S: Iterator<Item=char>> SetAnchor for NormalBuffer<S> {
    fn set_anchor(&mut self, anchor: Option<usize>) -> Option<usize> {
        self.buffer.set_anchor(anchor)
    }

    fn current_index(&mut self) -> usize {
        self.buffer.current_index()
    }

    fn revert(&mut self) {
        self.buffer.revert()
    }
}

/// A buffer with a custom anchor.
/// - When dropped, reset the anchor.
/// - When `revert` is called, reset `current` to the anchor.
pub struct AnchorBuffer<'a> {
    buffer: &'a mut dyn SetAnchor,
    anchor: Option<usize>,
}

impl<'a> AnchorBuffer<'a> {
    fn new(buffer: &'a mut impl SetAnchor) -> Self {
        let idx = buffer.current_index();
        AnchorBuffer { anchor: buffer.set_anchor(Some(idx)), buffer }
    }

    /// Undo all the read after the anchor was set.
    pub fn revert(&mut self) {
        self.buffer.revert();
    }
}

impl<'a> Drop for AnchorBuffer<'a> {
    fn drop(&mut self) {
        self.buffer.set_anchor(self.anchor);
    }
}

impl<'a> Buffer for AnchorBuffer<'a> {
    fn peek(&mut self) -> Option<char> {
        self.buffer.peek()
    }

    fn peek_n(&mut self, n: usize) -> buffer::Iter {
        self.buffer.peek_n(n)
    }

    fn next(&mut self) -> Option<char> {
        self.buffer.next()
    }

    fn next_n(&mut self, n: usize) -> buffer::Iter {
        self.buffer.next_n(n)
    }
}

impl<'a> SetAnchor for AnchorBuffer<'a> {
    fn set_anchor(&mut self, anchor: Option<usize>) -> Option<usize> {
        self.buffer.set_anchor(anchor)
    }

    fn current_index(&mut self) -> usize {
        self.buffer.current_index()
    }

    fn revert(&mut self) {
        self.buffer.revert()
    }
}

macro_rules! impl_anchor {
    () => {
        /// Set an anchor at the current reading position.
        pub fn anchor(&mut self) -> AnchorBuffer {
            AnchorBuffer::new(self)
        }
    }
}

impl<S: Iterator<Item=char>> NormalBuffer<S> { impl_anchor!(); }

impl<'a> AnchorBuffer<'a> { impl_anchor!(); }

/// Scanner with a back buffer.
pub struct Scanner<'a> {
    buffer: &'a mut dyn Buffer,
    location: Location,
}

impl<'a> Scanner<'a> {
    /// Create a new scanner from the back buffer.
    pub fn new(buffer: &'a mut impl Buffer) -> Self {
        Scanner { buffer, location: Location::new() }
    }
}

#[cfg(test)]
mod tests {
    mod normal_buffer {
        use crate::scanner::{NormalBuffer, Buffer};
        use crate::utils::LIPSUM;

        #[test]
        fn test_basics() {
            let mut buffer = NormalBuffer::new(LIPSUM.chars());
            assert_eq_str!(buffer.peek_n(5), LIPSUM[..5]);
            assert_eq_str!(buffer.next_n(5), LIPSUM[..5]);
            assert_eq_str!(buffer.peek_n(7), LIPSUM[5..5 + 7]);
            assert_eq_str!(buffer.buffer.iter(), LIPSUM[5..5 + 7]);
        }
    }
}
