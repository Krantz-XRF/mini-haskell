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

//! lexical scanner for mini-haskell

use crate::utils::*;
use crate::buffer::{Buffer, Stream, raw::Iter};
use crate::char::{CharPredicate, Maybe, Unicode};
use crate::token::LexemeType;
use crate::error::{DiagnosticEngine, DiagnosticMessage, DiagnosticReporter};
use crate::error::DiagnosticMessage::Error;
use crate::error::Error::{IncompleteLexeme, InvalidChar};
use crate::token::LexemeType::Whitespace;

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

    fn peek_n(&mut self, n: usize) -> Iter {
        self.buffer.peek_n(n)
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

    fn next_n(&mut self, _: usize) -> Iter {
        panic!("Never use Scanner::next_n, because location \
                information cannot be properly maintained")
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

    /// Haskell 2010 Report (2.2.whitespace)
    pub fn whitespace(&mut self) -> Result<()> {
        // whitespace -> whitestuff {whitestuff}
        self.some_(&mut method!(whitestuff))
    }

    fn whitestuff(&mut self) -> Result<()> {
        // whitestuff -> whitechar | comment | ncomment
        alt!(self, method!(whitechar), method!(comment), method!(ncomment));
        self.expected(Whitespace)
    }

    fn whitechar(&mut self) -> Option<()> {
        // whitechar  -> newline | vertab | space | tab | uniWhite
        // vertab     -> a vertical tab
        // space      -> a space
        // uniWhite   -> any Unicode character defined as whitespace
        simple_alt!(self,
            method!(newline), method!(tab),
            choice!(any!('\u{B}', ' ', Unicode::White)))
    }

    fn newline(&mut self) -> Option<()> {
        // newline    -> return linefeed | return | linefeed | formfeed
        // return     -> a carriage return
        // linefeed   -> a line feed
        // formfeed   -> a form feed
        let res = simple_alt!(self,
                choice!('\r', '\n'),
                choice!(any!('\r', '\n', '\u{C}')));
        if res.is_some() {
            self.location.newline();
        }
        res
    }

    fn tab(&mut self) -> Option<()> {
        // tab        -> a horizontal tab
        analyse!(self, '\t');
        self.location.tablise();
        Some(())
    }

    fn comment(&mut self) -> Option<()> {
        // comment    -> dashes [ any<symbol> {any} ] newline
        analyse!(self, '-', '-', *'-');
        if Unicode::Symbol.check(self.peek()?) { return None; }
        analyse!(self, *not!("\r\n\u{C}"));
        self.newline()
    }

    fn ncomment(&mut self) -> Option<()> {
        // ncomment   -> opencom ANYseq {ncomment ANYseq} closecom
        // opencom    -> {-
        // closecom   -> -}
        // ANYseq     -> {ANY}<{ANY} ( opencom | closecom ) {ANY}>
        // ANY        -> graphic | whitechar
        // any        -> graphic | space | tab
        // graphic    -> small | large | symbol | digit | special | " | '
        let begin = self.location;
        analyse!(self, '{', '-');
        const WHATEVER: char = '\u{0}';
        let mut last = WHATEVER;
        let mut depth = 1;
        while let Some(x) = self.next() {
            match (last, x) {
                ('-', '}') => {
                    last = x;
                    depth -= 1
                }
                ('{', '-') => {
                    last = WHATEVER;
                    depth += 1
                }
                _ => last = x,
            }
            if depth == 0 { break; }
        }
        if depth != 0 {
            let end = self.location;
            self.report(Error(IncompleteLexeme(Whitespace)))
                .within(begin, end)
        }
        Some(())
    }
}

#[cfg(test)]
mod tests {
    use crate::buffer::normal::NormalBuffer;
    use crate::scanner::Scanner;
    use crate::error::{DiagnosticEngine, Diagnostic};
    use crate::utils::setup_logger;

    fn test_scanner_on<U: Eq + std::fmt::Debug>(
        input: &str, f: impl for<'a> FnOnce(&'a mut Scanner<'a>) -> U,
        res: U, diags: &[Diagnostic]) {
        let mut buf = NormalBuffer::new(input.chars());
        let mut diag_engine = DiagnosticEngine::new();
        let mut scanner = Scanner::new(&mut buf, &mut diag_engine);
        assert_eq!(f(&mut scanner), res);
        assert_eq_iter!(diag_engine.iter(), diags.iter());
    }

    #[test]
    fn test_whitespace() {
        setup_logger();
        fn test(input: &str) {
            test_scanner_on(input, method!(whitestuff), Ok(()), &[]);
        }
        test("\r\n");
        test("\rA");
        test("\nB");
        test("--- Comment123!@#$%^&*()-=_+[]{}\\|;:'\",<.>/?`~\n");
        test("{- {--- AA -} B--}");
    }
}
