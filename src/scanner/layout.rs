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

use crate::scanner::{FatLexemeIterator, Range};
use crate::lexeme::{Lexeme, Lexeme::*};
use crate::scanner::layout::LastLexeme::StartOfFile;
use std::iter::Peekable;
use crate::lexeme::RId::Module;

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
    iterator: Peekable<&'a mut FatLexemeIterator<I>>,
    last_lexeme: LastLexeme,
    last_line: usize,
}

impl<'a, I: std::io::Read> From<&'a mut FatLexemeIterator<I>> for EnrichedLexemeIterator<'a, I> {
    fn from(iterator: &'a mut FatLexemeIterator<I>) -> Self {
        Self { iterator: iterator.peekable(), last_lexeme: StartOfFile, last_line: 0 }
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

#[cfg(test)]
mod tests {
    use indoc::indoc;
    use super::EnrichedLexemeIterator;
    use crate::scanner::FatLexemeIterator;
    // use crate::lexeme::Lexeme::{self, *};
    // use crate::lexeme::RId::*;
    // use crate::lexeme::ROp::*;

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
    fn test_lexeme_iterator() {
        let mut it = FatLexemeIterator::new(TEST_SOURCE.as_bytes());
        let mut enriched = EnrichedLexemeIterator::from(&mut it);
        for x in enriched.by_ref() {
            println!("{:?}", x);
        }
        let (err, _) = it.into_scanner();
        assert_eq!(err, None);
    }
}
