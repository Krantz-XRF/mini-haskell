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

use crate::utils::*;

/// Ring buffer (growable).
pub struct RingBuffer {
    /// back buffer.
    data: Vec<char>,
    /// next position to insert at, not valid till next insertion.
    front: usize,
    /// next position to read from, might be invalid.
    current: usize,
    /// position from `anchor` is potentially useful, thus cannot be overridden.
    anchor: Option<usize>,
}

/// Const iterator for traversing the ring buffer.
pub struct Iter<'a> {
    buffer: &'a RingBuffer,
    current_index: usize,
    boundary: usize,
}

impl<'a> Iterator for Iter<'a> {
    type Item = char;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_index < self.boundary {
            Some(self.buffer.at(inc(&mut self.current_index)))
        } else {
            None
        }
    }
}

/// Mutable iterator for traversing the ring buffer.
pub struct IterMut<'a> {
    buffer: &'a mut RingBuffer,
    current_index: usize,
    boundary: usize,
}

impl<'a> Iterator for IterMut<'a> {
    type Item = &'a mut char;

    fn next(&mut self) -> Option<Self::Item> {
        let buffer = unsafe_dup_mut!(self.buffer);
        if self.current_index < self.boundary {
            Some(buffer.at_mut(inc(&mut self.current_index)))
        } else {
            None
        }
    }
}

impl RingBuffer {
    /// Default buffer size for `RingBuffer`.
    pub const DEFAULT_BUFFER_SIZE: usize = 512;
    /// Minimum buffer growth at each buffer exhaustion.
    pub const MINIMUM_BUFFER_GROWTH: usize = 512;

    /// Create a new `RingBuffer` with `DEFAULT_BUFFER_SIZE`.
    pub fn new() -> Self { Self::new_sized(Self::DEFAULT_BUFFER_SIZE) }

    /// Create a new `RingBuffer` with specified size.
    pub fn new_sized(n: usize) -> Self {
        let n = round_to(n, Self::MINIMUM_BUFFER_GROWTH);
        RingBuffer {
            data: vec!['\0'; n],
            front: 0,
            current: 0,
            anchor: None,
        }
    }

    /// Whole buffer size.
    pub fn capacity(&self) -> usize {
        self.data.len()
    }

    /// Set a new anchor, and return the old one.
    pub fn set_anchor(&mut self, anchor: Option<usize>) -> Option<usize> {
        std::mem::replace(&mut self.anchor, anchor)
    }

    /// The current index, yet to be read from the buffer.
    pub fn current_index(&self) -> usize {
        self.current
    }

    /// Revert `current` to `anchor`. Panics if no anchor is set.
    pub fn revert(&mut self) {
        self.current = self.anchor.expect("cannot revert without a valid anchor.");
    }

    fn guard(&self) -> usize {
        std::cmp::min(self.current, self.anchor.unwrap_or(usize::MAX))
    }

    fn at(&self, n: usize) -> char {
        let sz = self.data.len();
        self.data[n % sz]
    }

    fn at_mut(&mut self, n: usize) -> &mut char {
        let sz = self.data.len();
        &mut self.data[n % sz]
    }

    /// Reserve space for at least `n` more characters.
    pub fn reserve_more(&mut self, n: usize) {
        let count = self.front - self.guard();
        // maximum characters after reserve, rounded.
        let sz = round_to(n + count, Self::MINIMUM_BUFFER_GROWTH);
        if self.data.len() >= sz { return (); }
        // allocate the new buffer.
        let buffer = std::mem::replace(&mut self.data, vec!['\0'; sz]);
        // copy all the data to the new buffer.
        let orig_sz = buffer.len();
        let orig_start = self.guard() % orig_sz;
        let start = self.guard() % sz;
        let (orig_shorter, len_min, len_max)
            = min_max(orig_sz - orig_start, sz - start);
        // (1) head: [.., start ~ start + len, ..]
        //     both are not crossing the border.
        let len = std::cmp::min(count, len_min);
        self.data[start..start + len].copy_from_slice(&buffer[orig_start..orig_start + len]);
        if len == count { return (); }
        // (2) body: [.., start + len ~ start + len2, ..]
        //     only one is crossing the border.
        let len2 = std::cmp::min(count, len_max);
        if orig_shorter {
            self.data[start + len..start + len2].copy_from_slice(&buffer[0..len2 - len]);
        } else {
            self.data[0..len2 - len].copy_from_slice(&buffer[orig_start + len..orig_start + len2]);
        }
        if len2 == count { return (); }
        // (3) tail: [.., start + len2 ~ end, ..]
        //     both are crossing the border.
        let orig_tail = (self.guard() + len2) % orig_sz;
        let tail = (self.guard() + len2) % sz;
        let orig_end = self.front % orig_sz;
        let end = self.front % sz;
        self.data[tail..end].copy_from_slice(&buffer[orig_tail..orig_end]);
    }

    /// Push 1 character into the buffer, without checking the bounds or growing the buffer.
    pub unsafe fn push_unchecked(&mut self, x: char) {
        *self.at_mut(self.front) = x;
        self.front += 1
    }

    /// Push 1 character into the buffer.
    pub fn push(&mut self, x: char) {
        self.reserve_more(1);
        unsafe { self.push_unchecked(x) }
    }

    /// Push no more than `n` characters from an iterator, return the number of characters
    /// successfully pushed to the buffer.
    ///
    /// Invariant: `buf.push_n(n, it) < n` if and only if `it` is exhausted.
    pub fn push_n(&mut self, n: usize, it: &mut impl Iterator<Item=char>) -> usize {
        self.reserve_more(n);
        for i in 0..n {
            if let Some(x) = it.next() {
                unsafe { self.push_unchecked(x) }
            } else {
                return i;
            }
        }
        n
    }

    /// Pop (possibly) 1 character from the buffer.
    pub fn pop(&mut self) -> Option<char> {
        let res = self.peek();
        self.current += 1;
        res
    }

    /// Pop no more than `n` characters from the buffer.
    pub fn pop_n(&mut self, n: usize) -> Iter {
        let current_index = self.current;
        self.current = std::cmp::min(self.current + n, self.front);
        Iter { current_index, boundary: self.current, buffer: self }
    }

    /// Peek (possibly) 1 character from the buffer.
    pub fn peek(&mut self) -> Option<char> {
        if self.current < self.front {
            Some(self.at(self.current))
        } else {
            None
        }
    }

    /// Peek no more than `n` characters from the buffer.
    pub fn peek_n(&mut self, n: usize) -> Iter {
        let boundary = std::cmp::min(self.current + n, self.front);
        Iter { current_index: self.current, boundary, buffer: self }
    }

    /// The number of remaining characters to read.
    pub fn remaining_count(&mut self) -> usize {
        self.front - self.current
    }

    /// Immutable traversal of ring buffer.
    pub fn iter(&self) -> Iter {
        Iter { current_index: self.current, boundary: self.front, buffer: self }
    }

    /// Mutable traversal of ring buffer.
    pub fn iter_mut(&mut self) -> IterMut {
        IterMut { current_index: self.current, boundary: self.front, buffer: self }
    }
}

#[cfg(test)]
mod tests {
    use super::RingBuffer;
    use crate::utils::LIPSUM;

    #[test]
    fn test_basics() {
        let mut buf = RingBuffer::new();
        let mut it = "Hello, world!".chars();
        assert_eq!(buf.push_n(5, &mut it), 5);
        assert_eq_str!(buf.iter(), "Hello");
        assert_eq_str!(it, ", world!");
        assert_eq_str!(buf.pop_n(2), "He");
        assert_eq_str!(buf.iter(), "llo");
        assert_eq!(buf.pop(), Some('l'));
        assert_eq_str!(buf.iter(), "lo");
    }

    #[test]
    fn test_growth() {
        let mut buf = RingBuffer::new();
        let mut it = LIPSUM.chars();

        buf.push_n(512, &mut it);
        assert_eq!(buf.data.len(), RingBuffer::DEFAULT_BUFFER_SIZE);
        assert_eq_iter!(buf.iter(), LIPSUM.chars().take(512));

        assert_eq_str!(buf.pop_n(41), LIPSUM[..41]);
        assert_eq_str!(buf.iter(), LIPSUM[41..512]);
        buf.push_n(23, &mut it);
        assert_eq!(buf.data.len(), RingBuffer::DEFAULT_BUFFER_SIZE);
        assert_eq_str!(buf.iter(), LIPSUM[41..512 + 23]);
        buf.push_n(67, &mut it);
        assert_eq!(buf.data.len(), RingBuffer::DEFAULT_BUFFER_SIZE * 2);
        assert_eq_str!(buf.iter(), LIPSUM[41..512 + 23 + 67]);

        buf.push_n(2000, &mut it);
        assert_eq_str!(buf.iter(), LIPSUM[41..]);
    }
}
