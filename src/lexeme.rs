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

//! Haskell lexemes.

/// Haskell lexeme types.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum LexemeType {
    /// Whitespaces.
    Whitespace,
    /// Identifiers.
    Identifier,
    /// Integers.
    Integer,
}

/// Haskell lexemes.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct Lexeme {
    /// text length of this lexeme, starting at the current position.
    pub length: usize,
    /// lexeme payload (contents).
    pub r#type: LexemeType,
}
