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

//! chars and strings: see "Haskell 2010 Report: 2.6 Character and String Literals".

use std::convert::identity;
use num_bigint::BigInt;
use num_traits::ToPrimitive;

use super::{Scanner, Result, numeric::Digit};
use crate::char::{Stream, CharPredicate, Ascii};
use crate::error::Diagnostic;
use crate::error::DiagnosticMessage::Error;
use crate::error::Error::CharOutOfBound;
use crate::lexeme::Lexeme::{self, CharLiteral, StringLiteral};

impl<I: std::io::Read> Scanner<I> {
    /// Character literals or string literals.
    pub fn char_or_string(&mut self) -> Result<Lexeme> {
        alt!(self, Self::char, Self::string);
        Self::keep_trying()
    }

    fn char(&mut self) -> Option<Lexeme> {
        // char     -> ' ( graphic<’ | \> | space | escape<\&> ) '
        analyse!(self, '\'');
        let c = simple_alt!(self, choice!(c; c: not!("'\\")), Self::escape)?;
        analyse!(self, '\'');
        Some(CharLiteral(c))
    }

    fn string(&mut self) -> Option<Lexeme> {
        // string   -> " {graphic<" | \>  | space | escape | gap} "
        analyse!(self, '"');
        let s = identity::<Option<_>>(self.many(
            |this| {
                alt!(this, seq!("\\&" => None),
                           choice!(Some(c); c: not!("\"\\")),
                           |this| this.escape().map(Some),
                           |this| this.gap().map(|_| None));
                None
            },
            String::new(),
            |res: &mut String, c| if let Some(c) = c { res.push(c) }))?;
        analyse!(self, '"');
        Some(StringLiteral(s))
    }

    fn escape(&mut self) -> Option<char> {
        // escape   -> \ ( charesc | ascii | decimal | o octal | x hexadecimal )
        analyse!(self, '\\');
        simple_alt!(self,
            Self::char_esc,
            Self::ascii,
            |this| this.numeric_escape(10),
            |this| { analyse!(this, 'o'); this.numeric_escape(8) },
            |this| { analyse!(this, 'x'); this.numeric_escape(16) })
    }

    fn numeric_escape(&mut self, base: u32) -> Option<char> {
        let start_loc = self.location;
        analyse!(self, d: {BigInt::from(0)}{Self::app_int(base)} +Digit);
        Some(d.to_u32().and_then(std::char::from_u32).unwrap_or_else(|| {
            Diagnostic::new(self.location, Error(CharOutOfBound(d)))
                .within(start_loc, self.location)
                .report(&mut self.diagnostics);
            '�'
        }))
    }

    fn char_esc(&mut self) -> Option<char> {
        // charesc  -> a | b | f | n | r | t | v | \ | " | ’ | &
        // Note: '\&' produces no character at all, so should be specially handled.
        Some(match self.next()? {
            'a' => '\u{7}',
            'b' => '\u{8}',
            'f' => '\u{C}',
            'n' => '\n',
            'r' => '\r',
            't' => '\t',
            'v' => '\u{B}',
            '\\' => '\\',
            '"' => '"',
            '\'' => '\'',
            _ => return None,
        })
    }

    fn gap(&mut self) -> Option<()> {
        // gap      -> \ whitechar {whitechar} \
        analyse!(self, '\\');
        identity::<Option<_>>(self.some_(Self::whitechar))?;
        analyse!(self, '\\');
        Some(())
    }

    fn ascii(&mut self) -> Option<char> {
        simple_alt!(self, Self::ascii_control, Self::ascii_rest)
    }

    fn ascii_control(&mut self) -> Option<char> {
        // cntrl    -> ascLarge | @ | [ | \ | ] | ˆ | _
        analyse!(self, '^');
        simple_alt!(self,
            choice!(char::from(c as u8 - b'A' + 1); c: Ascii::Upper),
            choice!('\0'; '@'),
            choice!(char::from(27); '['),
            choice!(char::from(28); '\\'),
            choice!(char::from(29); ']'),
            choice!(char::from(30); '^'),
            choice!(char::from(31); '_'))
    }

    fn ascii_rest(&mut self) -> Option<char> {
        // ascii    -> ˆcntrl | NUL | SOH | STX | ETX | EOT | ENQ | ACK
        //           | BEL | BS | HT | LF | VT | FF | CR | SO | SI | DLE
        //           | DC1 | DC2 | DC3 | DC4 | NAK | SYN | ETB | CAN
        //           | EM | SUB | ESC | FS | GS | RS | US | SP | DEL
        let names = ["NUL", "SOH", "STX", "ETX", "EOT", "ENQ", "ACK",
            "BEL", "BS", "HT", "LF", "VT", "FF", "CR", "SO", "SI", "DLE",
            "DC1", "DC2", "DC3", "DC4", "NAK", "SYN", "ETB", "CAN",
            "EM", "SUB", "ESC", "FS", "GS", "RS", "US", "SP", "DEL"];
        for (k, nm) in names.iter().copied().enumerate() {
            if let Some(r) = self.anchored(seq!(nm => k)) {
                return Some(char::from(r as u8));
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use crate::scanner::test_scanner_on;
    use crate::utils::setup_logger;
    use crate::utils::Result3::Success;
    use crate::lexeme::Lexeme::{self, CharLiteral, StringLiteral};

    #[test]
    fn test_char_string() {
        setup_logger();
        fn test(input: &str, res: Lexeme) {
            trace!(scanner, "test on {:?} ...", input);
            test_scanner_on(input, method!(char_or_string), Success(res), None);
        }
        test("'A'", CharLiteral('A'));
        test(r"'\r'", CharLiteral('\r'));
        test(r"'\ESC'", CharLiteral('\x1b'));
        test(r"'\^X'", CharLiteral('\x18'));
        test(r#""A\r\ESC\^X""#, StringLiteral("A\r\x1b\x18".to_string()));
        test(r#""\SO\&H\SOH\4\&2\
                      \Some\&Other\nText""#,
             StringLiteral("\x0eH\x01\x042SomeOther\nText".to_string()));
    }
}
