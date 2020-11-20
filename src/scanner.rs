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

pub mod identifier;
pub mod whitespace;

use crate::utils::*;
use crate::input::Input;
use crate::lexeme::LexemeType;
use crate::char::{CharPredicate, Maybe, Unicode, Stream};
use crate::error::{
    Diagnostic, DiagnosticsEngine, DiagnosticMessage::Error,
    Error::{InvalidUTF8, InputFailure, InvalidChar},
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
pub struct Scanner<I> {
    input: Input<I>,
    location: Location,
    diagnostics: DiagnosticsEngine,
}

impl<I: std::io::Read> Stream for Scanner<I> {
    fn peek(&mut self) -> Option<char> {
        match self.input.clone().next(|s| Diagnostic::new(
            self.location, Error(InvalidUTF8(Vec::from(s))))
            .report(&mut self.diagnostics)) {
            Ok((c, _)) => Some(c),
            Err(_) => None,
        }
    }

    fn next(&mut self) -> Option<char> {
        let res = self.next_input();
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
                Diagnostic::new(self.location, Error(InvalidChar(x)))
                    .report(&mut self.diagnostics);
            }
        }
        res
    }
}

impl<I: std::io::Read> Scanner<I> {
    fn next_input(&mut self) -> Option<char> {
        let diagnostics = &mut self.diagnostics;
        let location = self.location;
        match self.input.clone().next(move |s| Diagnostic::new(
            location, Error(InvalidUTF8(Vec::from(s))))
            .report(diagnostics))
            .map_err(Into::into) {
            Ok((c, rest)) => {
                self.input = rest;
                Some(c)
            }
            Err(e) => {
                if let Some(e) = e {
                    Diagnostic::new(self.location, Error(InputFailure(e)))
                        .report(&mut self.diagnostics);
                }
                None
            }
        }
    }

    /// Pop many characters until the predicate fails.
    pub fn span<T>(&mut self, mut f: impl FnMut(char) -> bool,
                   init: T, mut join: impl FnMut(&mut T, char)) -> T {
        let mut res = init;
        while let Some(x) = self.peek() {
            if !f(x) { break; }
            join(&mut res, x);
            self.next();
        }
        res
    }

    /// Pop many characters until the predicate fails, collect them into a `Vec`.
    pub fn span_collect(&mut self, f: impl FnMut(char) -> bool) -> Vec<char> {
        self.span(f, Vec::new(), Vec::push)
    }

    /// Pop many characters until the predicate fails, ignore the characters.
    pub fn span_(&mut self, f: impl FnMut(char) -> bool) {
        self.span(f, (), |_, _| ())
    }

    /// Fail with `t` as the expected token type.
    pub fn expected<T>(&mut self, t: LexemeType) -> Result<T> {
        Err(LexError { expected: t, unexpected: self.peek() })
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

impl<I> Scanner<I> {
    /// Create a new scanner from the back buffer.
    pub fn new(input: I) -> Self {
        Scanner {
            input: Input::new(input),
            location: Location::new(),
            diagnostics: DiagnosticsEngine::new(),
        }
    }

    /// Set an anchor for possible revert in future. Use `Result` for error indication.
    pub fn anchored<R: Maybe>(&mut self, f: impl FnOnce(&mut Scanner<I>) -> R) -> R {
        let old_input = self.input.clone();
        let old_location = self.location;
        let old_diagnostics_count = self.diagnostics.len();
        let res = f(self);
        if res.is_nothing() {
            self.input = old_input;
            self.location = old_location;
            self.diagnostics.truncate(old_diagnostics_count);
        }
        res
    }

    /// Match many of this rule.
    pub fn many<T, U>(
        &mut self, f: &mut impl FnMut(&mut Scanner<I>) -> Result<T>,
        init: U, join: &mut impl FnMut(&mut U, T)) -> Result<U> {
        let mut res = init;
        while let Ok(x) = f(self) {
            join(&mut res, x);
        }
        Ok(res)
    }

    /// Match many of this rule, ignore the results.
    pub fn many_<T>(&mut self, f: &mut impl FnMut(&mut Scanner<I>) -> Result<T>) -> Result<()> {
        self.many(f, (), &mut |_, _| ())
    }

    /// Match many of this rule.
    pub fn some<T, U>(
        &mut self, f: &mut impl FnMut(&mut Scanner<I>) -> Result<T>,
        mut init: U, join: &mut impl FnMut(&mut U, T)) -> Result<U> {
        join(&mut init, f(self)?);
        self.many(f, init, join)
    }

    /// Match many of this rule, ignore the results.
    pub fn some_<T>(&mut self, f: &mut impl FnMut(&mut Scanner<I>) -> Result<T>) -> Result<()> {
        self.some(f, (), &mut |_, _| ())
    }
}
