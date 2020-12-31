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

pub mod basic;
pub mod identifier;
pub mod whitespace;
pub mod numeric;
pub mod char_string;
pub mod special;
pub mod layout;

use crate::utils::*;
use crate::input::Input;
use crate::lexeme::{LexemeType, Lexeme};
use crate::char::{CharPredicate, Stream};
use crate::error::{
    Diagnostic, DiagnosticsEngine, DiagnosticMessage::Error,
    Error::{InvalidUTF8, InputFailure, InvalidChar},
};
use crate::utils::Result3::{FailFast, RetryLater, Success};
use crate::scanner::basic::Any;

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
            if !Any.check(x) {
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

    /// Fail fast with `t` as the expected lexeme type.
    pub fn expected<T>(&mut self, t: LexemeType) -> Result<T> {
        FailFast(self.err_expected(t))
    }

    /// Fail for future recovery from `alt!`.
    pub fn keep_trying<T>() -> Result<T> { RetryLater(()) }

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
pub type Result<T> = crate::utils::Result3<T, LexError, ()>;

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
        while let Ok(x) = self.anchored(&mut f).into_result() {
            join(&mut res, x);
        }
        Either::right(res)
    }

    /// Match many of this rule, ignore the results.
    pub fn many_<ET: Either<Left=E>, ER: Either<Left=E>, E>(
        &mut self, f: impl FnMut(&mut Scanner<I>) -> ET) -> ER
        where ER::Right: From<()> {
        let unit: ER::Right = From::from(());
        self.many(f, unit, |_, _| ())
    }

    /// Match many of this rule.
    pub fn some<ET: Either<Left=E>, EU: Either<Left=E>, E>(
        &mut self, mut f: impl FnMut(&mut Scanner<I>) -> ET,
        mut init: EU::Right, mut join: impl FnMut(&mut EU::Right, ET::Right)) -> EU {
        join(&mut init, unwrap!(f(self)));
        self.many(f, init, join)
    }

    /// Match many of this rule, ignore the results.
    pub fn some_<ET: Either<Left=E>, ER: Either<Left=E>, E>(
        &mut self, f: impl FnMut(&mut Scanner<I>) -> ET) -> ER
        where ER::Right: From<()> {
        let unit: ER::Right = From::from(());
        self.some(f, unit, |_, _| ())
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
    pub fn sep_by_<ET, EU, ER, E>(
        &mut self,
        f: impl FnMut(&mut Scanner<I>) -> ET,
        g: impl FnMut(&mut Scanner<I>) -> ER) -> EU
        where ET: Either<Left=E>,
              ER: Either<Left=E>,
              EU: Either<Left=E>,
              EU::Right: From<()> {
        let unit: EU::Right = From::from(());
        self.sep_by(f, g, unit, |_, _| ())
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

impl<I: std::io::Read> Scanner<I> {
    /// Get the next lexeme from the [`Scanner`].
    pub fn next_lexeme(&mut self) -> Result<Lexeme> {
        alt!(self, Self::numeric_literal,
                   Self::id_or_sym,
                   Self::char_or_string,
                   Self::special);
        Self::keep_trying()
    }
}

#[cfg(test)]
fn test_scanner_on<U: Eq + std::fmt::Debug>(
    input: &str,
    f: impl FnOnce(&mut Scanner<&[u8]>) -> U,
    res: U, next: Option<char>) {
    let mut scanner = Scanner::new(input.as_bytes());
    assert_eq!(f(&mut scanner), res);
    assert_eq!(scanner.next(), next);
}

/// An iterator of lexemes from an [`Input`] stream.
pub struct RawLexemeIterator<I: std::io::Read> {
    scanner: Scanner<I>,
    error: Option<LexError>,
}

impl<I: std::io::Read> Iterator for RawLexemeIterator<I> {
    type Item = Lexeme;
    fn next(&mut self) -> Option<Lexeme> {
        self.enriched_next(|_| ()).map(|t| t.0)
    }
}

impl<I: std::io::Read> From<Scanner<I>> for RawLexemeIterator<I> {
    fn from(scanner: Scanner<I>) -> Self {
        Self {
            error: None,
            scanner,
        }
    }
}

impl<I: std::io::Read> RawLexemeIterator<I> {
    /// Create a new lexeme iterator from raw input.
    pub fn new(input: I) -> Self { Self::from(Scanner::new(input)) }
    /// Get back the internal scanner of this iterator.
    pub fn into_scanner(self) -> (Option<LexError>, Scanner<I>) { (self.error, self.scanner) }
    fn enriched_next<T>(&mut self, proc: impl FnOnce(&Scanner<I>) -> T) -> Option<(Lexeme, T)> {
        if self.error.is_some() { return None; }
        // possibly consume whitespaces and ignore errors.
        let _ = self.scanner.whitespace();
        // for the fat iterator to insert a statement to get the location.
        let val = proc(&mut self.scanner);
        // produce a lexeme.
        match self.scanner.next_lexeme() {
            Success(x) => Some((x, val)),
            RetryLater(_) => None,
            FailFast(err) => {
                self.error = Some(err);
                None
            }
        }
    }
}

/// A "fat" lexeme iterator, i.e. iterator for lexemes with their location ranges.
pub struct FatLexemeIterator<I: std::io::Read> {
    iterator: RawLexemeIterator<I>,
    location: Location,
}

impl<I: std::io::Read> Iterator for FatLexemeIterator<I> {
    type Item = (Lexeme, Range);
    fn next(&mut self) -> Option<(Lexeme, Range)> {
        let (x, location) = self.iterator.enriched_next(|s| s.location)?;
        self.location = location;
        Some((x, Range {
            begin: location,
            end: self.iterator.scanner.location,
        }))
    }
}

impl<I: std::io::Read> From<RawLexemeIterator<I>> for FatLexemeIterator<I> {
    fn from(iterator: RawLexemeIterator<I>) -> Self {
        Self {
            location: iterator.scanner.location,
            iterator,
        }
    }
}

impl<I: std::io::Read> FatLexemeIterator<I> {
    /// Create a new lexeme iterator from raw input.
    pub fn new(input: I) -> Self { Self::from(RawLexemeIterator::<I>::new(input)) }
    /// Get back the internal scanner of this iterator.
    pub fn into_scanner(self) -> (Option<LexError>, Scanner<I>) { self.iterator.into_scanner() }
}

#[cfg(test)]
mod tests {
    use indoc::indoc;
    use super::RawLexemeIterator;
    use crate::lexeme::Lexeme::{self, *};
    use crate::lexeme::RId::*;
    use crate::lexeme::ROp::*;

    const TEST_SOURCE: &str = indoc! {r#"
        module Main where
        import Prelude hiding (Integer)
        main :: IO ()
        main = do
            name <- getLine
            putStrLn ("Hello, " <> name <> "!")
            pure ()
    "#};

    fn expected_lexemes() -> Box<[Lexeme]> {
        vec![
            ReservedId(Module),
            Identifier("Main".to_string()),
            ReservedId(Where),
            ReservedId(Import),
            Identifier("Prelude".to_string()),
            Identifier("hiding".to_string()),
            OpenParenthesis,
            Identifier("Integer".to_string()),
            CloseParenthesis,
            Identifier("main".to_string()),
            ReservedOp(ColonColon),
            Identifier("IO".to_string()),
            OpenParenthesis,
            CloseParenthesis,
            Identifier("main".to_string()),
            ReservedOp(EqualSign),
            ReservedId(Do),
            Identifier("name".to_string()),
            ReservedOp(LeftArrow),
            Identifier("getLine".to_string()),
            Identifier("putStrLn".to_string()),
            OpenParenthesis,
            StringLiteral("Hello, ".to_string()),
            Operator("<>".to_string()),
            Identifier("name".to_string()),
            Operator("<>".to_string()),
            StringLiteral("!".to_string()),
            CloseParenthesis,
            Identifier("pure".to_string()),
            OpenParenthesis,
            CloseParenthesis,
        ].into_boxed_slice()
    }

    #[test]
    fn test_lexeme_iterator() {
        let mut it = RawLexemeIterator::new(TEST_SOURCE.as_bytes());
        assert!(it.by_ref().eq(expected_lexemes().iter().cloned()));
        let (err, _) = it.into_scanner();
        assert_eq!(err, None);
    }
}
