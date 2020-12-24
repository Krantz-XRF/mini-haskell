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

//! basic character classes in "Haskell 2010 Report, 2.2 Lexical Program Structure".

use crate::char::{Ascii, Unicode};

alias! {
    /// see "Haskell 2010 Report, 2.2 Lexical Program Structure".
    /// ```text
    /// small       -> ascSmall | uniSmall | _
    /// ascSmall    -> a | b | ... | z
    /// uniSmall    -> any Unicode lowercase letter
    /// ```
    pub Small = any!(Ascii::Lower, Unicode::Lower, '_');

    /// see "Haskell 2010 Report, 2.2 Lexical Program Structure".
    /// ```text
    /// large       -> ascLarge | uniLarge
    /// ascLarge    -> A | B | ... | Z
    /// uniLarge    -> any uppercase or titlecase Unicode letter
    /// ```
    pub Large = any!(Ascii::Upper, Unicode::Upper);

    /// see "Haskell 2010 Report, 2.2 Lexical Program Structure".
    /// ```text
    /// symbol      -> ascSymbol | uniSymbol<special | _ | " | '>
    /// ascSymbol   -> ! | # | $ | % | & | * | + | . | / | < | = | > | ? | @
    ///              | \ | ^ | | | - | ~ | :
    /// uniSymbol   -> any Unicode symbol or punctuation
    /// special     -> ( | ) | , | ; | [ | ] | ` | { | }
    /// ```
    pub Symbol = any!(r"!#$%&*+./<=>?@\^|-~:",
                      all!(any!(Unicode::Symbol, Unicode::Punct),
                           not!("_\"'"), not!(Special)));

    /// see "Haskell 2010 Report, 2.2 Lexical Program Structure".
    /// ```text
    /// graphic     -> small | large | symbol | digit | special | " | '
    /// ```
    pub Graphic = any!(Small, Large, Symbol, Digit, Special, '"', '\'');

    /// see "Haskell 2010 Report, 2.2 Lexical Program Structure".
    /// ```text
    /// special     -> ( | ) | , | ; | [ | ] | ` | { | }
    /// ```
    pub Special = "(),;[]`{}";

    /// see "Haskell 2010 Report, 2.2 Lexical Program Structure".
    /// ```text
    /// digit       -> ascDigit | uniDigit
    /// ascDigit    -> 0 | 1 | ... | 9
    /// uniDigit    -> any Unicode decimal digit
    /// ```
    /// TODO: Properly handle Unicode digits.
    pub Digit = any!(Ascii::Digit, Unicode::Digit);

    /// see "Haskell 2010 Report, 2.2 Lexical Program Structure".
    /// ```text
    /// octit       -> 0 | 1 | ... | 7
    /// ```
    pub Octit = '0'..='7';

    /// see "Haskell 2010 Report, 2.2 Lexical Program Structure".
    /// ```text
    /// hexit       -> digit | A | ... | F | a | ... | f
    /// ```
    pub Hexit = any!(Digit, 'A'..='F', 'a'..='f');

    /// see "Haskell 2010 Report, 2.2 Lexical Program Structure".
    /// ```text
    /// whitechar   -> newline | vertab | space | tab | uniWhite
    /// vertab      -> a vertical tab
    /// space       -> a space
    /// uniWhite    -> any Unicode character defined as whitespace
    /// newline     -> return linefeed | return | linefeed | formfeed
    /// return      -> a carriage return
    /// linefeed    -> a line feed
    /// formfeed    -> a form feed
    /// ```
    pub WhiteChar = any!("\r\n\u{C}\u{B} \t", Unicode::White);

    /// see "Haskell 2010 Report, 2.2 Lexical Program Structure".
    /// ```text
    /// ANY         -> graphic | whitechar
    /// ```
    pub Any = any!(Graphic, WhiteChar);
}
