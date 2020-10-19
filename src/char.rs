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

//! character related utilities

/// ASCII character categories.
pub enum Ascii {
    /// Any ASCII
    Any,
    /// ASCII letters: `[A-Za-z]`.
    Alpha,
    /// ASCII letters and digits: `[A-Za-z0-9]`.
    AlphaNum,
    /// ASCII digits: `[0-9]`.
    Digit,
    /// ASCII octal digits: `[0-7]`.
    Oct,
    /// ASCII hex digits: `[0-9a-zA-Z]`.
    Hex,
    /// ASCII lowercase letters: `[a-z]`.
    Lower,
    /// ASCII uppercase letters: `[A-Z]`.
    Upper,
    /// ASCII whitespaces: `[ \t\r\n\v]`.
    White,
}

/// Unicode character categories.
pub enum Unicode {
    /// Unicode letters: `Alphabetic`.
    Alpha,
    /// Unicode letter and numeric.
    AlphaNum,
    /// Unicode numeric: `Nd`, `Nl`, `No`.
    Numeric,
    /// Unicode lowercase letters: `Lowercase`.
    Lower,
    /// Unicode uppercase letters: `Uppercase`.
    Upper,
    /// Unicode whitespaces: `White_Space`.
    White,
}

/// Anything that can be used as a character predicate.
pub trait CharPredicate {
    /// Check whether the character is in this category.
    fn check(&self, x: char) -> bool;
}

impl CharPredicate for Ascii {
    fn check(&self, x: char) -> bool {
        match self {
            Ascii::Any => x.is_ascii(),
            Ascii::Alpha => x.is_ascii_alphabetic(),
            Ascii::AlphaNum => x.is_ascii_alphanumeric(),
            Ascii::Digit => x.is_ascii_digit(),
            Ascii::Oct => x.is_digit(8),
            Ascii::Hex => x.is_ascii_hexdigit(),
            Ascii::Lower => x.is_ascii_lowercase(),
            Ascii::Upper => x.is_ascii_uppercase(),
            Ascii::White => x.is_ascii_whitespace(),
        }
    }
}

impl CharPredicate for Unicode {
    fn check(&self, x: char) -> bool {
        match self {
            Unicode::Alpha => x.is_alphabetic(),
            Unicode::AlphaNum => x.is_alphanumeric(),
            Unicode::Numeric => x.is_numeric(),
            Unicode::Lower => x.is_lowercase(),
            Unicode::Upper => x.is_uppercase(),
            Unicode::White => x.is_whitespace(),
        }
    }
}

impl CharPredicate for char {
    fn check(&self, x: char) -> bool { *self == x }
}

impl CharPredicate for str {
    fn check(&self, x: char) -> bool { self.contains(x) }
}

#[allow(unused_macros)]
macro_rules! analyse {
    ($lexer: expr) => {};
    ($lexer: expr, $x: ident : $($rest: tt)+) => {
        check_and_continue_analyse!(once, nodrop, $lexer, $x, $($rest)+);
    };
    ($lexer: expr, $x: ident + : $($rest: tt)+) => {
        check_and_continue_analyse!(some, nodrop, $lexer, $x, $($rest)+);
    };
    ($lexer: expr, $x: ident * : $($rest: tt)+) => {
        check_and_continue_analyse!(many, nodrop, $lexer, $x, $($rest)+);
    };
    ($lexer: expr, $($rest: tt)+) => {
        check_and_continue_analyse!(once, drop, $lexer, _x, $($rest)+);
    };
}

#[allow(unused_macros)]
macro_rules! drop_and_analyse {
    (drop $x: ident, $($rest: tt)*) => {
        std::mem::drop($x);
        analyse!($($rest)*);
    };
    (nodrop $x: ident, $($rest: tt)*) => {
        analyse!($($rest)*);
    }
}

#[allow(unused_macros)]
macro_rules! check_and_continue_analyse {
    ($count: ident, $drop: ident, $lexer: expr, $x: ident,
     any($($($params: tt)+)?) $(, $($rest: tt)+)?) => {
        check_once_many_some!($lexer, $count, $x, check_any!($x $(, $($params)+)?));
        drop_and_analyse!($drop $x, $lexer $(, $($rest)+)?);
    };
    ($count: ident, $drop: ident, $lexer: expr, $x: ident,
     all($($($params: tt)+)?) $(, $($rest: tt)+)?) => {
        check_once_many_some!($lexer, $count, $x, check_all!($x $(, $($params)+)?));
        drop_and_analyse!($drop $x, $lexer $(, $($rest)+)?);
    };
    ($count: ident, $drop: ident, $lexer: expr, $x: ident, $m: expr $(, $($rest: tt)+)?) => {
        check_once_many_some!($lexer, $count, $x, $m.check($x));
        drop_and_analyse!($drop $x, $lexer $(, $($rest)+)?);
    }
}

#[allow(unused_macros)]
macro_rules! check_once_many_some {
    ($lexer: expr, once, $x: ident, $cond: expr) => {
        let $x = $lexer.next()?;
        if !$cond { return None; }
        let $x = $x;
    };
    ($lexer: expr, many, $x: ident, $cond: expr) => {
        let (_, $x) = $lexer.span(&mut |$x| { $cond })?;
    };
    ($lexer: expr, some, $x: ident, $cond: expr) => {
        let (_n, $x) = $lexer.span(&mut |$x| { $cond })?;
        if _n == 0 { return None; }
    };
}

#[allow(unused_macros)]
macro_rules! check_any {
    ($x: ident) => { false };
    ($x: ident, any($($($params: tt)+)?) $(, $($rest: tt)+)?) => {
        check_any!($x $(, $($params)+)? $(, $($rest)+)?)
    };
    ($x: ident, all($($($params: tt)+)?) $(, $($rest: tt)+)?) => {
        check_all!($x $(, $($params)+)?) || check_any!($x $(, $($rest)+)?)
    };
    ($x: ident, $m: expr $(, $($rest: tt)+)?) => {
        $m.check($x) || check_any!($x $(, $($rest)+)?)
    }
}

#[allow(unused_macros)]
macro_rules! check_all {
    ($x: ident) => { true };
    ($x: ident, any($($($params: tt)+)?) $(, $($rest: tt)+)?) => {
        check_any!($x $(, $($params)+)?) && check_all!($x $(, $($rest)+)?)
    };
    ($x: ident, all($($($params: tt)+)?) $(, $($rest: tt)+)?) => {
        check_all!($x $(, $($params)+)? $(, $($rest)+)?)
    };
    ($x: ident, $m: expr $(, $($rest: tt)+)?) => {
        $m.check($x) && check_all!($x $(, $($rest)+)?)
    }
}

#[cfg(test)]
mod tests {
    use super::{Unicode, Ascii, CharPredicate};
    use crate::buffer::Buffer;

    #[test]
    fn test_syntax() {
        #[allow(dead_code)]
        #[allow(unused_variables)]
        fn parse(scanner: &mut impl Buffer) -> Option<()> {
            analyse!(scanner);
            analyse!(scanner, x: Ascii::Any);
            analyse!(scanner, x+: Unicode::Alpha, '\n');
            analyse!(scanner, x*: any(Unicode::Alpha, Ascii::Digit));
            analyse!(scanner, x: "aeiou");
            Some(())
        }
    }
}
