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

//! anchor buffers, i.e. normal buffers with anchors.

use super::{raw, Buffer};

/// A buffer with a custom anchor.
/// - When dropped, reset the anchor.
/// - When `revert` is called, reset `current` to the anchor.
pub struct AnchorBuffer<'a> {
    buffer: &'a mut dyn Buffer,
    anchor: Option<usize>,
}

impl<'a> AnchorBuffer<'a> {
    pub(super) fn new(buffer: &'a mut dyn Buffer) -> Self {
        let idx = buffer.current_index();
        AnchorBuffer { anchor: buffer.set_anchor(Some(idx)), buffer }
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

    fn peek_n(&mut self, n: usize) -> raw::Iter {
        self.buffer.peek_n(n)
    }

    fn next(&mut self) -> Option<char> {
        self.buffer.next()
    }

    fn next_n(&mut self, n: usize) -> raw::Iter {
        self.buffer.next_n(n)
    }

    fn set_anchor(&mut self, anchor: Option<usize>) -> Option<usize> {
        self.buffer.set_anchor(anchor)
    }

    fn current_index(&mut self) -> usize {
        self.buffer.current_index()
    }

    fn revert(&mut self) {
        self.buffer.revert()
    }

    impl_buffer_common!();
}

#[cfg(test)]
mod tests {
    use crate::utils::LIPSUM;
    use crate::buffer::normal::NormalBuffer;
    use crate::buffer::{Buffer};

    #[test]
    fn test_basics() {
        let mut buffer = NormalBuffer::new(LIPSUM.chars());
        assert_eq_str!(buffer.next_n(42), LIPSUM[..42]);
        /* anchored here! */ {
            let mut buffer = buffer.anchor();
            assert_eq_str!(buffer.next_n(304), LIPSUM[42..42 + 304]);
            // buffer not reverted
        }
        assert_eq!(buffer.current_index(), 42 + 304);
        /* anchored here! */ {
            let mut buffer = buffer.anchor();
            assert_eq_str!(buffer.next_n(211), LIPSUM[42 + 304..42 + 304 + 211]);
            // buffer reverted
            buffer.revert();
        }
        assert_eq!(buffer.current_index(), 42 + 304);
    }
}
