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

pub mod raw;
pub mod normal;
pub mod anchor;

/// A back buffer for a scanner.
pub trait Buffer {
    /// Peek the next character without consuming it.
    fn peek(&mut self) -> Option<char>;
    /// Peek no more than `n` characters without consuming them.
    fn peek_n(&mut self, n: usize) -> raw::Iter;
    /// Take the next character and consume it.
    fn next(&mut self) -> Option<char>;
    /// Take no more than `n` characters and consume them.
    fn next_n(&mut self, n: usize) -> raw::Iter;
}

trait SetAnchor: Buffer {
    fn set_anchor(&mut self, anchor: Option<usize>) -> Option<usize>;
    fn current_index(&mut self) -> usize;
    fn revert(&mut self);
}
