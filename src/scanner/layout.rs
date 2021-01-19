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
use crate::scanner::layout::AugmentedLexeme::{PhantomCloseCurlyBracket, PhantomSemicolon, PhantomOpenCurlyBracket, Real};
use crate::utils::iter::IterStream;
use std::collections::VecDeque;

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

impl From<(Lexeme, Range)> for EnrichedLexeme {
    fn from((lexeme, range): (Lexeme, Range)) -> Self {
        EnrichedLexeme::Normal(lexeme, range)
    }
}

/// Lexeme stream enriched with `{n}` and `<n>`.
/// See "Haskell 2010 Report, 10.3 Layout".
pub struct EnrichedLexemeIterator<I: std::io::Read> {
    iterator: IterStream<FatLexemeIterator<I>>,
    last_lexeme: LastLexeme,
    last_line: usize,
}

impl<I: std::io::Read> EnrichedLexemeIterator<I> {
    /// Create a new enriched lexeme iterator from raw input.
    pub fn new(input: I) -> Self { Self::from(FatLexemeIterator::<I>::new(input)) }
    /// Get back the internal scanner of this iterator.
    pub fn into_scanner(self) -> (Option<LexError>, Scanner<I>) { self.iterator.unwrap().into_scanner() }
}

impl<I: std::io::Read> From<FatLexemeIterator<I>> for EnrichedLexemeIterator<I> {
    fn from(iterator: FatLexemeIterator<I>) -> Self {
        Self {
            iterator: IterStream::from(iterator),
            last_lexeme: LastLexeme::StartOfFile,
            last_line: 0,
        }
    }
}

impl<I: std::io::Read> Iterator for EnrichedLexemeIterator<I> {
    type Item = EnrichedLexeme;
    fn next(&mut self) -> Option<Self::Item> {
        use LastLexeme::*;
        use EnrichedLexeme::*;
        let next = self.iterator.peek(0);
        match self.last_lexeme {
            // If a `let`, `where`, `do`, or `of` keyword is not followed by the lexeme `{`
            LetWhereDoOf if next.is_none() || next.unwrap().0 != OpenCurlyBracket => {
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

/// Augmented lexemes: normal lexemes or phantom `{`s, `;`s, and `}`s.
pub enum AugmentedLexeme {
    /// Real lexemes.
    Real(Lexeme, Range),
    /// Phantom `{`.
    PhantomOpenCurlyBracket,
    /// Phantom `}`.
    PhantomCloseCurlyBracket,
    /// Phantom `;`.
    PhantomSemicolon,
}

impl Display for AugmentedLexeme {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Real(t, range) => write!(f, "{}: {}", range, t),
            PhantomOpenCurlyBracket => write!(f, "<phantom>: {{"),
            PhantomCloseCurlyBracket => write!(f, "<phantom>: }}"),
            PhantomSemicolon => write!(f, "<phantom>: ;"),
        }
    }
}

/// Lexeme streams augmented with phantom `{`, `;`, and `}`.
pub struct AugmentedLexemeIterator<I: std::io::Read> {
    iterator: IterStream<EnrichedLexemeIterator<I>>,
    indents: Vec<usize>,
    buffer: VecDeque<AugmentedLexeme>,
}

impl<'a, I: std::io::Read> AugmentedLexemeIterator<I> {
    /// Create a new enriched lexeme iterator from raw input.
    pub fn new(input: I) -> Self { Self::from(EnrichedLexemeIterator::new(input)) }
    /// Get back the internal scanner of this iterator.
    pub fn into_scanner(self) -> (Option<LexError>, Scanner<I>) { self.iterator.unwrap().into_scanner() }

    fn prepare_next(&mut self) {
        let t = self.iterator.next();
        // L [] []                = []
        // L [] (m : ms)          = } : L [] ms if m /= 0 (Note 6)
        // Note 6. At the end of the input, any pending close-braces are inserted.
        // It is an error at this point to be within a non-layout context (i.e. m = 0).
        if t.is_none() {
            if let Some(k) = self.indents.pop() {
                if k == 0 { panic!("mismatched curly brackets.") }
                self.buffer.push_back(PhantomCloseCurlyBracket)
            }
            return;
        }
        use EnrichedLexeme::*;
        match (t.unwrap(), self.indents.last().copied()) {
            // L (<n>: ts) (m : ms)   = ; : (L ts (m : ms)) if m = n
            //                        = } : (L (<n>: ts) ms) if n < m
            (AngleN(n), Some(m)) if m == n =>
                self.buffer.push_back(PhantomSemicolon),
            (AngleN(n), Some(m)) if n < m => {
                self.iterator.put_back(AngleN(n));
                self.indents.pop();
                self.buffer.push_back(PhantomCloseCurlyBracket)
            }
            // L (<n>: ts) ms         = L ts ms
            (AngleN(_), _) => self.prepare_next(),
            // L ({n} : ts) (m : ms)  = { : (L ts (n : m : ms)) if n > m (Note 1)
            // L ({n} : ts) []        = { : (L ts [n]) if n > 0 (Note 1)
            (CurlyN(n), m) if m.is_none() || n > m.unwrap() => {
                self.indents.push(n);
                self.buffer.push_back(PhantomOpenCurlyBracket)
            }
            // L ({n} : ts) ms        = { : } : (L (<n>: ts) ms) (Note 2)
            (CurlyN(n), _) => {
                self.buffer.push_back(PhantomOpenCurlyBracket);
                self.buffer.push_back(PhantomCloseCurlyBracket);
                self.iterator.put_back(AngleN(n))
            }
            // L (} : ts) (0 : ms)    = } : (L ts ms) (Note 3)
            // L (} : ts) ms          = parse-error (Note 3)
            // Note 3.By matching against 0 for the current layout context, we ensure that an
            // explicit close brace can only match an explicit open brace. A parse error results
            // if an explicit close brace matches an implicit open brace.
            (Normal(CloseCurlyBracket, loc), Some(k)) => {
                assert_eq!(k, 0, "mismatched curly brackets.");
                self.indents.pop();
                self.buffer.push_back(Real(CloseCurlyBracket, loc))
            }
            // L ({ : ts) ms          = { : (L ts (0 : ms)) (Note 4)
            (Normal(OpenCurlyBracket, loc), _) => {
                self.indents.push(0);
                self.buffer.push_back(Real(OpenCurlyBracket, loc))
            }
            // L (t : ts) (m : ms)    = } : (L (t : ts) ms) if m /= 0 and parse-error(t) (Note 5)
            // TODO: implement this `parse-error(t)` rule.
            // L (t : ts) ms          = t : (L ts ms)
            (Normal(t, loc), _) => {
                self.buffer.push_back(Real(t, loc))
            }
        }
    }
}

impl<'a, I: std::io::Read> From<EnrichedLexemeIterator<I>> for AugmentedLexemeIterator<I> {
    fn from(iterator: EnrichedLexemeIterator<I>) -> Self {
        AugmentedLexemeIterator {
            iterator: IterStream::from(iterator),
            buffer: VecDeque::new(),
            indents: Vec::new(),
        }
    }
}

impl<'a, I: std::io::Read> Iterator for AugmentedLexemeIterator<I> {
    type Item = AugmentedLexeme;
    fn next(&mut self) -> Option<AugmentedLexeme> {
        self.prepare_next();
        self.buffer.pop_front()
    }
}

#[cfg(test)]
mod tests {
    use indoc::indoc;
    use super::RawLexemeIterator;
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
        use expect_test::expect;
        let mut it = EnrichedLexemeIterator::new(TEST_SOURCE.as_bytes());
        let mut res = String::new();
        for t in it.by_ref() { res += &format!("{}\n", t) }
        expect![[r#"
            1:1-1:7: module
            1:8-1:12: Main
            1:13-1:18: where
            {1}
            2:1-2:7: import
            2:8-2:15: Prelude
            2:16-2:22: hiding
            2:23-2:24: (
            2:24-2:31: Integer
            2:31-2:32: )
            <1>
            3:1-3:5: main
            3:6-3:8: ::
            3:9-3:11: IO
            3:12-3:13: (
            3:13-3:14: )
            <1>
            4:1-4:5: main
            4:6-4:7: =
            4:8-4:10: do
            {5}
            5:5-5:9: name
            5:10-5:12: <-
            5:13-5:20: getLine
            <5>
            6:5-6:13: putStrLn
            6:14-6:15: (
            6:15-6:24: "Hello, "
            6:25-6:27: <>
            6:28-6:32: name
            6:33-6:35: <>
            6:36-6:39: "!"
            6:39-6:40: )
            <5>
            7:5-7:9: pure
            7:10-7:11: (
            7:11-7:12: )
        "#]].assert_eq(&res);
        let (err, _) = it.into_scanner();
        assert_eq!(err, None);
    }
}
