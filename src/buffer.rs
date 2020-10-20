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

//! ring buffer for characters.

macro_rules! impl_buffer_common {
    () => {
        /// Set an anchor at the current reading position.
        fn anchor(&mut self) -> $crate::buffer::anchor::AnchorBuffer {
            $crate::buffer::anchor::AnchorBuffer::new(self)
        }
    }
}

pub mod raw;
pub mod normal;
pub mod anchor;

/// A continuous text stream.
pub trait Stream {
    /// Peek the next character without consuming it.
    fn peek(&mut self) -> Option<char>;
    /// Peek no more than `n` characters without consuming them.
    fn peek_n(&mut self, n: usize) -> raw::Iter;
    /// Take the next character and consume it.
    fn next(&mut self) -> Option<char>;
    /// Take no more than `n` characters and consume them.
    fn next_n(&mut self, n: usize) -> raw::Iter;
}

/// A back buffer for a scanner.
pub trait Buffer: Stream {
    /// Revert the buffer to its anchor.
    /// Panics if no anchor is present.
    fn revert(&mut self);
    /// Set an anchor at the current reading position.
    fn anchor(&mut self) -> anchor::AnchorBuffer;

    #[doc(hidden)]
    fn set_anchor(&mut self, anchor: Option<usize>) -> Option<usize>;
    #[doc(hidden)]
    fn current_index(&mut self) -> usize;
}

/// Peek many characters until the predicate fails.
pub fn span(this: &mut impl Stream, mut f: impl FnMut(char) -> bool) -> Option<Vec<char>> {
    let mut buf = Vec::new();
    while let Some(x) = this.next() {
        if !f(x) { break; }
        buf.push(x);
    }
    Some(buf)
}
