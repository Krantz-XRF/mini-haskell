/*
 * mini-haskell: light-weight Haskell for fun
 * Copyright (C) 2021  Xie Ruifeng
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

//! Iterator utilities.

use std::collections::VecDeque;

/// Make a stream from an iterator.
pub struct IterStream<I: Iterator> {
    raw_iter: I,
    buffer: VecDeque<I::Item>,
}

impl<I: Iterator> From<I> for IterStream<I> {
    fn from(raw_iter: I) -> Self {
        IterStream {
            raw_iter,
            buffer: VecDeque::new(),
        }
    }
}

impl<I: Iterator> Iterator for IterStream<I> {
    type Item = I::Item;
    fn next(&mut self) -> Option<I::Item> {
        self.buffer.pop_front().or_else(|| self.raw_iter.next())
    }
}

impl<I: Iterator> IterStream<I> {
    /// Put one item back to the stream.
    pub fn put_back(&mut self, x: I::Item) {
        self.buffer.push_front(x)
    }

    fn prepare(&mut self, n: usize) -> Option<()> {
        while self.buffer.len() < n {
            self.buffer.push_back(self.raw_iter.next()?)
        }
        Some(())
    }

    /// Peek the nth element without consuming it.
    pub fn peek(&mut self, n: usize) -> Option<&I::Item> {
        self.prepare(n)?;
        Some(&self.buffer[n])
    }

    /// Unwraps the [`IterStream`] and get back the underlying iterator.
    /// # Panics
    /// Panics if there are items already peeked but not consumed yet.
    pub fn unwrap(self) -> I {
        assert!(self.buffer.is_empty());
        self.raw_iter
    }

    /// Unwraps the [`IterStream`] and get back the underlying iterator
    /// and, if any, items already peeked but not consumed yet.
    pub fn unwrap_full(self) -> (I, impl IntoIterator<Item=I::Item>) {
        (self.raw_iter, self.buffer)
    }

    /// Begin the multi-peek mode.
    pub fn multi_peek(&mut self) -> IterStreamMultiPeek<I> {
        IterStreamMultiPeek {
            iter_stream: self,
            current_position: 0,
        }
    }
}

/// A special [`IterStream`] where consecutive `peek`s return consecutive items from the stream.
pub struct IterStreamMultiPeek<'a, I: Iterator> {
    iter_stream: &'a mut IterStream<I>,
    current_position: usize,
}

impl<'a, I: Iterator> IterStreamMultiPeek<'a, I> {
    /// Peek the next element and advance the internal cursor.
    pub fn peek_ref(&mut self) -> Option<&I::Item> {
        let res = self.iter_stream.peek(self.current_position);
        self.current_position += 1;
        res
    }
}

impl<'a, I: Iterator> IterStreamMultiPeek<'a, I> where I::Item: Copy {
    /// Peek the next element and advance the internal cursor.
    pub fn peek(&mut self) -> Option<I::Item> {
        self.peek_ref().copied()
    }
}
