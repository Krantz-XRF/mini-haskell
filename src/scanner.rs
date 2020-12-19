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
use crate::char::{CharPredicate, Unicode, Stream};
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

    fn r#match<'a>(&mut self, s: &'a str) -> Option<&'a str> {
        self.input.clone().r#match(s, |s|
            Diagnostic::new(self.location, Error(InvalidUTF8(Vec::from(s))))
                .report(&mut self.diagnostics),
        ).map(|rest| {
            self.input = rest;
            s
        })
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

    /// Fail with `t` as the expected lexeme type.
    pub fn expected<T>(&mut self, t: LexemeType) -> Result<T> {
        Err(self.err_expected(t))
    }

    /// Create a `LexError` with the expected lexeme type.
    pub fn err_expected(&mut self, t: LexemeType) -> LexError {
        LexError { expected: t, unexpected: self.peek() }
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

    /// Set an anchor for possible revert in future. Use an `Either` for error indication.
    pub fn anchored<R: Either>(&mut self, f: impl FnOnce(&mut Scanner<I>) -> R) -> R {
        let old_input = self.input.clone();
        let old_location = self.location;
        let old_diagnostics_count = self.diagnostics.len();
        match f(self).into_result() {
            Ok(res) => Either::right(res),
            Err(err) => {
                self.input = old_input;
                self.location = old_location;
                self.diagnostics.truncate(old_diagnostics_count);
                Either::left(err)
            }
        }
    }

    /// Match many of this rule.
    pub fn many<ET: Either<Left=E>, EU: Either<Left=E>, E>(
        &mut self, mut f: impl FnMut(&mut Scanner<I>) -> ET,
        init: EU::Right, mut join: impl FnMut(&mut EU::Right, ET::Right)) -> EU {
        let mut res = init;
        while let Ok(x) = f(self).into_result() {
            join(&mut res, x);
        }
        Either::right(res)
    }

    /// Match many of this rule, ignore the results.
    pub fn many_<ET: Either<Left=E>, ER: Either<Left=E, Right=()>, E>(
        &mut self, f: impl FnMut(&mut Scanner<I>) -> ET) -> ER {
        self.many(f, (), |_, _| ())
    }

    /// Match many of this rule.
    pub fn some<ET: Either<Left=E>, EU: Either<Left=E>, E>(
        &mut self, mut f: impl FnMut(&mut Scanner<I>) -> ET,
        mut init: EU::Right, mut join: impl FnMut(&mut EU::Right, ET::Right)) -> EU {
        join(&mut init, unwrap!(f(self)));
        self.many(f, init, join)
    }

    /// Match many of this rule, ignore the results.
    pub fn some_<ET: Either<Left=E>, ER: Either<Left=E, Right=()>, E>(
        &mut self, f: impl FnMut(&mut Scanner<I>) -> ET) -> ER {
        self.some(f, (), |_, _| ())
    }

    /// Match many of this rule separated by some other rule.
    pub fn sep_by<ET: Either<Left=E>, EU: Either<Left=E>, ER: Either<Left=E>, E>(
        &mut self,
        mut f: impl FnMut(&mut Scanner<I>) -> ET,
        mut g: impl FnMut(&mut Scanner<I>) -> ER,
        mut init: EU::Right, mut join: impl FnMut(&mut EU::Right, ET::Right)) -> EU {
        join(&mut init, unwrap!(f(self)));
        self.many(move |scanner| {
            unwrap!(g(scanner));
            f(scanner)
        }, init, join)
    }

    /// Match many of this rule separated by some other rule, ignore the results.
    pub fn sep_by_<ET: Either<Left=E>, EU: Either<Left=E, Right=()>, ER: Either<Left=E>, E>(
        &mut self,
        f: impl FnMut(&mut Scanner<I>) -> ET,
        g: impl FnMut(&mut Scanner<I>) -> ER) -> EU {
        self.sep_by(f, g, (), |_, _| ())
    }

    /// Match many of this rule ended by some other rule.
    pub fn end_by<ET: Either<Left=E>, EU: Either<Left=E>, ER: Either<Left=E>, E>(
        &mut self,
        mut f: impl FnMut(&mut Scanner<I>) -> ET,
        mut g: impl FnMut(&mut Scanner<I>) -> ER,
        init: EU::Right, join: impl FnMut(&mut EU::Right, ET::Right)) -> EU {
        self.many(move |scanner| scanner.anchored(|scanner| {
            let res = f(scanner);
            unwrap!(g(scanner));
            res
        }), init, join)
    }
}
