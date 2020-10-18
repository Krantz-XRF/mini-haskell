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
use crate::buffer::Buffer;

/// Source location.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
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

/// Scanner with a back buffer.
pub struct Scanner<'a> {
    buffer: &'a mut dyn buffer::Buffer,
    location: Location,
}

impl<'a> Scanner<'a> {
    /// Create a new scanner from the back buffer.
    pub fn new(buffer: &'a mut impl buffer::Buffer) -> Self {
        Scanner { buffer, location: Location::new() }
    }

    /// Set an anchor for possible revert in future.
    pub fn anchor<T>(&mut self, f: impl FnOnce(&mut Scanner) -> Option<T>) -> Option<T> {
        let mut anchored = self.buffer.anchor();
        let mut scanner = Scanner { location: self.location, buffer: &mut anchored };
        let res = f(&mut scanner);
        if res.is_none() { anchored.revert() }
        res
    }
}
