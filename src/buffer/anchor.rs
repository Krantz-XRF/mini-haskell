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

use super::{raw, normal, Buffer, SetAnchor};

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

    fn peek_n(&mut self, n: usize) -> raw::Iter {
        self.buffer.peek_n(n)
    }

    fn next(&mut self) -> Option<char> {
        self.buffer.next()
    }

    fn next_n(&mut self, n: usize) -> raw::Iter {
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

impl<S: Iterator<Item=char>> normal::NormalBuffer<S> { impl_anchor!(); }

impl<'a> AnchorBuffer<'a> { impl_anchor!(); }
