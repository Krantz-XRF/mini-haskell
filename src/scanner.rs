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

//! lexical scanner for mini-haskell.

pub mod whitespace;

use crate::utils::*;
use crate::buffer::{Buffer, Stream};
use crate::char::{CharPredicate, Maybe, Unicode};
use crate::token::LexemeType;
use crate::error::{
    DiagnosticEngine,
    DiagnosticReporter,
    DiagnosticMessage,
    DiagnosticMessage::Error,
    Error::InvalidChar,
};

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
    /// Size of a Tab stop.
    pub const TAB_SIZE: usize = 8;

    /// Create a new location, the same as `Location::default()`.
    pub fn new() -> Self { Self::default() }

    /// Step one character.
    pub fn step(&mut self) {
        self.column += 1;
        self.offset += 1;
    }

    /// Start a new line.
    pub fn newline(&mut self) {
        self.column = 0;
        self.line += 1;
    }

    /// Align to the next tab stop.
    pub fn tablise(&mut self) {
        self.step();
        self.column = round_to(self.column, Self::TAB_SIZE);
    }
}

/// A half-open source range: a pair of `Location`s.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct Range {
    /// Where the range begins (inclusive).
    pub begin: Location,
    /// Where the range ends (non-inclusive).
    pub end: Location,
}

/// Scanner with a back buffer.
pub struct Scanner<'a> {
    buffer: &'a mut dyn Buffer,
    location: Location,
    diagnostics: &'a mut DiagnosticEngine,
}

impl<'a> Stream for Scanner<'a> {
    fn peek(&mut self) -> Option<char> {
        self.buffer.peek()
    }

    fn next(&mut self) -> Option<char> {
        let res = self.buffer.next();
        if let Some(x) = res {
            self.location.step();
            // ANY        -> graphic | whitechar
            // graphic    -> small | large | symbol | digit | special | " | '
            // special    -> ( | ) | , | ; | [ | ] | ` | { | }
            // symbol     -> ascSymbol | uniSymbol<special | _ | " | '>
            // ascSymbol  -> ! | # | $ | % | & | * | + | . | / | < | = | > | ? | @
            //             | \ | ^ | | | - | ~ | :
            // uniSymbol  -> any Unicode symbol or punctuation
            if !any!("(),;[]`{}\"\'\r\n\t\u{B}\u{C}", Unicode::White,
                     Unicode::Lower, Unicode::Upper,
                     r"!#$%&*+./<=>?@\^|-~:",
                     Unicode::Symbol, Unicode::Punct,
                     Unicode::Digit, "(),;[]`{}").check(x) {
                self.report(Error(InvalidChar(x)));
            }
        }
        res
    }
}

/// Lexical error.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct LexError {
    /// The expected lexeme type at the error.
    pub expected: LexemeType,
    /// The character at which tokenization fails.
    pub unexpected: Option<char>,
}

/// Lexer result.
pub type Result<T> = std::result::Result<T, LexError>;

impl<'a> Scanner<'a> {
    /// Create a new scanner from the back buffer.
    pub fn new(buffer: &'a mut impl Buffer, diagnostics: &'a mut DiagnosticEngine) -> Self {
        Scanner { buffer, location: Location::new(), diagnostics }
    }

    /// Set an anchor for possible revert in future. Use `Result` for error indication.
    pub fn anchored<R: Maybe>(&mut self, f: impl FnOnce(&mut Scanner) -> R) -> R {
        let old_location = self.location;
        let mut anchored = self.buffer.anchor();
        let mut scanner = Scanner {
            location: self.location,
            buffer: &mut anchored,
            diagnostics: &mut self.diagnostics,
        };
        let res = f(&mut scanner);
        if res.is_nothing() {
            anchored.revert();
            self.location = old_location;
        }
        res
    }

    /// Match many of this rule.
    pub fn many<T, U>(
        &mut self, f: &mut impl FnMut(&mut Scanner) -> Result<T>,
        init: U, join: &mut impl FnMut(&mut U, T)) -> Result<U> {
        let mut res = init;
        while let Ok(x) = f(self) {
            join(&mut res, x);
        }
        Ok(res)
    }

    /// Match many of this rule, ignore the results.
    pub fn many_<T>(&mut self, f: &mut impl FnMut(&mut Scanner) -> Result<T>) -> Result<()> {
        self.many(f, (), &mut |_, _| ())
    }

    /// Match many of this rule.
    pub fn some<T, U>(
        &mut self, f: &mut impl FnMut(&mut Scanner) -> Result<T>,
        mut init: U, join: &mut impl FnMut(&mut U, T)) -> Result<U> {
        join(&mut init, f(self)?);
        self.many(f, init, join)
    }

    /// Match many of this rule, ignore the results.
    pub fn some_<T>(&mut self, f: &mut impl FnMut(&mut Scanner) -> Result<T>) -> Result<()> {
        self.some(f, (), &mut |_, _| ())
    }

    /// Emit a diagnostic.
    pub fn report(&mut self, msg: DiagnosticMessage) -> DiagnosticReporter {
        self.diagnostics.report(self.location, msg)
    }

    /// Fail with `t` as the expected token type.
    pub fn expected<T>(&mut self, t: LexemeType) -> Result<T> {
        Err(LexError { expected: t, unexpected: self.peek() })
    }
}
