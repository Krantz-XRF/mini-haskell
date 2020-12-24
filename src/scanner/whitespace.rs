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

use super::{Result, Scanner};
use crate::char::{CharPredicate, Unicode, Stream};
use crate::error::{DiagnosticMessage::Error, Error::IncompleteLexeme, Diagnostic};
use crate::lexeme::LexemeType::Whitespace;

impl<I: std::io::Read> Scanner<I> {
    /// Haskell 2010 Report (2.2.whitespace)
    pub fn whitespace(&mut self) -> Result<()> {
        // whitespace -> whitestuff {whitestuff}
        self.some_(method!(whitestuff))
    }

    fn whitestuff(&mut self) -> Result<()> {
        // whitestuff -> whitechar | comment | ncomment
        alt!(self, method!(whitechar), method!(comment), method!(ncomment));
        Self::keep_trying()
    }

    pub(super) fn whitechar(&mut self) -> Option<()> {
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
            Diagnostic::new(self.location, Error(IncompleteLexeme(Whitespace)))
                .within(begin, end).report(&mut self.diagnostics)
        }
        Some(())
    }
}

#[cfg(test)]
mod tests {
    use crate::scanner::test_scanner_on;
    use crate::utils::setup_logger;
    use crate::utils::Result3::Success;

    #[test]
    fn test_whitespace() {
        setup_logger();
        fn test(input: &str) {
            test_scanner_on(input, method!(whitestuff), Success(()), None);
        }
        test("\r\n");
        test("\r");
        test("\n");
        test("--- Comment123!@#$%^&*()-=_+[]{}\\|;:'\",<.>/?`~\n");
        test("{- {--- AA -} B--}");
    }
}
