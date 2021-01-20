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

//! error reporting for the mini-Haskell compiler.

use num_bigint::BigInt;
use crate::lexeme::LexemeType;

/// An exhaustive list of compiler errors.
#[derive(Debug)]
pub enum Error {
    /// An invalid UTF-8 sequence.
    InvalidUTF8(Vec<u8>),
    /// A failure during the input process.
    InputFailure(std::io::Error),
    /// A Unicode character not accepted by the Haskell language.
    InvalidChar(char),
    // An error during the tokenization process.
    // InvalidToken(LexError),
    /// A lexeme ended prematurely, e.g. EOF in a block comment.
    IncompleteLexeme(LexemeType),
    /// A float literal is too large (or small) to represent.
    ///
    /// **Note**:
    ///
    /// - maximum value for IEEE 754 64-bit double is approximately 1.8e308;
    /// - to prevent loss of precision, maximum value to store in IEEE 754 64-bit double is 2^53;
    /// - `Rational` with an exponent 4096 takes approximately 13.3KiB to store;
    /// - large float literals may eventually exhaust the usable memory of the compiler;
    /// - `Rational` is probably not a good representation for large floats;
    FloatOutOfBound(BigInt),
    /// A character/string literal contains a Unicode character out of bound.
    CharOutOfBound(BigInt),
}

/// A diagnostic message (body).
#[derive(Debug)]
pub enum DiagnosticMessage {
    /// Critical errors.
    Error(Error),
}
