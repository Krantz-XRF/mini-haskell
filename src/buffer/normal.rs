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

//! normal buffers, basically a raw buffer with an associated input iterator.

use crate::utils::*;
use super::{raw, Buffer};

/// Buffer tied with an input iterator.
pub struct NormalBuffer<S: Iterator<Item=char>> {
    buffer: raw::RingBuffer,
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
        NormalBuffer { buffer: raw::RingBuffer::new(), input }
    }
}

impl<S: Iterator<Item=char>> Buffer for NormalBuffer<S> {
    fn peek(&mut self) -> Option<char> {
        self.read_n(1);
        self.buffer.peek()
    }

    fn peek_n(&mut self, n: usize) -> raw::Iter {
        self.read_n(n);
        self.buffer.peek_n(n)
    }

    fn next(&mut self) -> Option<char> {
        self.read_n(1);
        self.buffer.pop()
    }

    fn next_n(&mut self, n: usize) -> raw::Iter {
        self.read_n(n);
        self.buffer.pop_n(n)
    }

    fn revert(&mut self) {
        self.buffer.revert()
    }

    fn set_anchor(&mut self, anchor: Option<usize>) -> Option<usize> {
        self.buffer.set_anchor(anchor)
    }

    fn current_index(&mut self) -> usize {
        self.buffer.current_index()
    }

    impl_buffer_common!();
}

#[cfg(test)]
mod tests {
    use super::{NormalBuffer, Buffer};
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
