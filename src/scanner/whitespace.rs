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

//! whitespaces: see "Haskell 2010 Report: 2.2 Lexical Program Structure" and
//! "Haskell 2010 Report: 2.3 Comments".

use super::{Scanner, Result};
use crate::lexeme::LexemeType::Whitespace;
use crate::char::{Unicode, CharPredicate};
use crate::buffer::Stream;
use crate::error::{DiagnosticMessage::Error, Error::IncompleteLexeme};

impl<'a> Scanner<'a> {
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
    use crate::buffer::Stream;

    fn test_scanner_on<U: Eq + std::fmt::Debug>(
        input: &str, f: impl for<'a, 'b> FnOnce(&'a mut Scanner<'b>) -> U,
        res: U, next: Option<char>, diags: &[Diagnostic]) {
        let mut buf = NormalBuffer::new(input.chars());
        let mut diag_engine = DiagnosticEngine::new();
        let mut scanner = Scanner::new(&mut buf, &mut diag_engine);
        assert_eq!(f(&mut scanner), res);
        assert_eq!(scanner.next(), next);
        assert_eq_iter!(diag_engine.iter(), diags.iter());
    }

    #[test]
    fn test_whitespace() {
        setup_logger();
        fn test(input: &str) {
            test_scanner_on(input, method!(whitestuff), Ok(()), None, &[]);
        }
        test("\r\n");
        test("\r");
        test("\n");
        test("--- Comment123!@#$%^&*()-=_+[]{}\\|;:'\",<.>/?`~\n");
        test("{- {--- AA -} B--}");
    }
}

