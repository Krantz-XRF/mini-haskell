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
use std::fmt::{Display, Formatter};

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

impl Display for EnrichedLexeme {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        use EnrichedLexeme::*;
        match self {
            CurlyN(n) => write!(f, "{{{}}}", n),
            AngleN(n) => write!(f, "<{}>", n),
            Normal(lexeme, range) => write!(f, "{}: {}", range, lexeme)
        }
    }
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

/// An iterator of lexemes from an [`Input`](crate::input::Input) stream.
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
    use super::EnrichedLexemeIterator;
    use crate::lexeme::Lexeme::*;
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

    #[test]
    fn test_raw_iterator() {
        let mut it = RawLexemeIterator::new(TEST_SOURCE.as_bytes());
        assert!(it.by_ref().eq([
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
        ].iter().cloned()));
        let (err, _) = it.into_scanner();
        assert_eq!(err, None);
    }

    #[test]
    fn test_enriched_iterator() {
        let mut it = FatLexemeIterator::new(TEST_SOURCE.as_bytes());
        assert!(EnrichedLexemeIterator::from(&mut it)
            .map(|t| format!("{}", t)).eq([
            "1:1-1:7: module",
            "1:8-1:12: Main",
            "1:13-1:18: where",
            "{0}",
            "2:0-2:6: import",
            "2:7-2:14: Prelude",
            "2:15-2:21: hiding",
            "2:22-2:23: (",
            "2:23-2:30: Integer",
            "2:30-2:31: )",
            "<0>",
            "3:0-3:4: main",
            "3:5-3:7: ::",
            "3:8-3:10: IO",
            "3:11-3:12: (",
            "3:12-3:13: )",
            "<0>",
            "4:0-4:4: main",
            "4:5-4:6: =",
            "4:7-4:9: do",
            "{4}",
            "5:4-5:8: name",
            "5:9-5:11: <-",
            "5:12-5:19: getLine",
            "<4>",
            "6:4-6:12: putStrLn",
            "6:13-6:14: (",
            "6:14-6:23: \"Hello, \"",
            "6:24-6:26: <>",
            "6:27-6:31: name",
            "6:32-6:34: <>",
            "6:35-6:38: \"!\"",
            "6:38-6:39: )",
            "<4>",
            "7:4-7:8: pure",
            "7:9-7:10: (",
            "7:10-7:11: )",
        ].iter().copied()));
        let (err, _) = it.into_scanner();
        assert_eq!(err, None);
    }
}
