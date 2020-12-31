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

//! Haskell layout: see "Haskell 2010 Report, 10.3 Layout".

use super::{Range, LexError, Scanner, Location};
use crate::lexeme::{Lexeme, Lexeme::*, RId::Module};
use crate::utils::Result3::*;

enum LastLexeme {
    LetWhereDoOf,
    StartOfFile,
    // this means we have already handled the following lexeme.
    PassThrough,
    Other,
}

/// Enriched lexemes: a normal lexeme, a `{n}`, or an `<n>`.
#[derive(Debug, Eq, PartialEq)]
pub enum EnrichedLexeme {
    /// a `{n}`.
    CurlyN(usize),
    /// an `<n>`.
    AngleN(usize),
    /// a normal lexeme with a source range.
    Normal(Lexeme, Range),
}

/// Lexeme stream enriched with `{n}` and `<n>`.
/// See "Haskell 2010 Report, 10.3 Layout".
pub struct EnrichedLexemeIterator<'a, I: std::io::Read> {
    iterator: std::iter::Peekable<&'a mut FatLexemeIterator<I>>,
    last_lexeme: LastLexeme,
    last_line: usize,
}

impl<'a, I: std::io::Read> From<&'a mut FatLexemeIterator<I>> for EnrichedLexemeIterator<'a, I> {
    fn from(iterator: &'a mut FatLexemeIterator<I>) -> Self {
        Self { iterator: iterator.peekable(), last_lexeme: LastLexeme::StartOfFile, last_line: 0 }
    }
}

impl<'a, I: std::io::Read> Iterator for EnrichedLexemeIterator<'a, I> {
    type Item = EnrichedLexeme;
    fn next(&mut self) -> Option<Self::Item> {
        use LastLexeme::*;
        use EnrichedLexeme::*;
        let next = self.iterator.peek();
        match self.last_lexeme {
            // If a `let`, `where`, `do`, or `of` keyword is not followed by the lexeme `{`
            LetWhereDoOf if next.is_some() && next.unwrap().0 != OpenCurlyBracket => {
                self.last_lexeme = PassThrough;
                // where n is the indentation of the next lexeme if there is one
                // or 0 if the end of file has been reached
                let n = next.map_or(0, |t| t.1.begin.column);
                // the token `{n}` is inserted after the keyword
                Some(CurlyN(n))
            }
            // If the first lexeme of a module is not `{` or `module`
            StartOfFile if next.is_some()
                && ![OpenCurlyBracket, ReservedId(Module)]
                .contains(&next.unwrap().0) => {
                self.last_lexeme = PassThrough;
                // where n is the indentation of the lexeme
                let n = next.unwrap().1.begin.column;
                // then it is preceded by `{n}`
                Some(CurlyN(n))
            }
            // Where the start of a lexeme is preceded only by white space on the same line
            // provided that it is not, as a consequence of the first two rules, preceded by `{n}`
            Other if next.is_some() && next.unwrap().1.begin.line > self.last_line => {
                self.last_line = next.unwrap().1.begin.line;
                // where n is the indentation of the lexeme
                let n = next.unwrap().1.begin.column;
                // this lexeme is preceded by `<n>`
                Some(AngleN(n))
            }
            // otherwise we just return the normal lexeme
            _ => {
                let (lexeme, range) = self.iterator.next()?;
                // update last line for "preceded only by white space on the same line" test
                self.last_line = range.end.line;
                // update last lexeme for "4 keywords not followed by {" test
                use crate::lexeme::Lexeme::ReservedId as R;
                use crate::lexeme::RId::*;
                self.last_lexeme = match lexeme {
                    R(Let) | R(Where) | R(Do) | R(Of) => LetWhereDoOf,
                    _ => Other,
                };
                // return as a normal lexeme
                Some(Normal(lexeme, range))
            }
        }
    }
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
    use super::FatLexemeIterator;
    use crate::lexeme::Lexeme::{self, *};
    use crate::lexeme::RId::*;
    use crate::lexeme::ROp::*;
    use crate::scanner::layout::EnrichedLexemeIterator;

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
    fn test_raw_iterator() {
        let mut it = RawLexemeIterator::new(TEST_SOURCE.as_bytes());
        assert!(it.by_ref().eq(expected_lexemes().iter().cloned()));
        let (err, _) = it.into_scanner();
        assert_eq!(err, None);
    }

    #[test]
    fn test_enriched_iterator() {
        let mut it = FatLexemeIterator::new(TEST_SOURCE.as_bytes());
        let mut enriched = EnrichedLexemeIterator::from(&mut it);
        for x in enriched.by_ref() {
            println!("{:?}", x);
        }
        let (err, _) = it.into_scanner();
        assert_eq!(err, None);
    }
}
