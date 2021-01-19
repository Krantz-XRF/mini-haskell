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

//! special: see "Haskell 2010 Report: 2.2 Lexical Program Structure".

use super::{Scanner, Result};
use crate::utils::char::{Stream, CharPredicate};
use crate::lexeme::Lexeme::{self, *};

impl<I: std::io::Read> Scanner<I> {
    /// Special: delimiters.
    pub fn special(&mut self) -> Result<Lexeme> {
        alt!(self, choice!(Comma; ','),
                   choice!(Semicolon; ';'),
                   choice!(Backtick; '`'),
                   choice!(OpenCurlyBracket; '{'),
                   choice!(CloseCurlyBracket; '}'),
                   choice!(OpenParenthesis; '('),
                   choice!(CloseParenthesis; ')'),
                   choice!(OpenSquareBracket; '['),
                   choice!(CloseSquareBracket;']'));
        Self::keep_trying()
    }
}
