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

//! character related utilities.

use unic_ucd_category::GeneralCategory;

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
    /// Unicode decimal digits: `Nd`.
    Digit,
    /// Unicode lowercase letters: `Lowercase`.
    Lower,
    /// Unicode uppercase letters: `Uppercase`.
    Upper,
    /// Unicode whitespaces: `White_Space`.
    White,
    /// Unicode symbol: `Sm`, `Sc`, `Sk`, `So`.
    Symbol,
    /// Unicode punctuation: `Pc`, `Pd`, `Ps`, `Pe`, `Pi`, `Pf`.
    Punct,
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
            Unicode::Digit => GeneralCategory::of(x) == GeneralCategory::DecimalNumber,
            Unicode::Lower => x.is_lowercase(),
            Unicode::Upper => x.is_uppercase(),
            Unicode::White => x.is_whitespace(),
            Unicode::Symbol => GeneralCategory::of(x).is_symbol(),
            Unicode::Punct => GeneralCategory::of(x).is_punctuation(),
        }
    }
}

impl CharPredicate for char {
    fn check(&self, x: char) -> bool {
        *self == x
    }
}

/// A character range (half open), used as a candidate for `CharPredicate`.
///
/// ```
/// # use mini_haskell::char::CharPredicate;
/// assert_eq!(('a' .. 'z').check('a'), true);
/// assert_eq!(('a' .. 'z').check('z'), false);
/// ```
pub type CharRange = std::ops::Range<char>;

/// A character range (closed), used as a candidate for `CharPredicate`.
///
/// ```
/// # use mini_haskell::char::CharPredicate;
/// assert_eq!(('a' ..= 'z').check('a'), true);
/// assert_eq!(('a' ..= 'z').check('z'), true);
/// assert_eq!(('a' ..= 'z').check('3'), false);
/// ```
pub type CharRangeInclusive = std::ops::RangeInclusive<char>;

impl CharPredicate for CharRange {
    fn check(&self, x: char) -> bool {
        self.contains(&x)
    }
}

impl CharPredicate for CharRangeInclusive {
    fn check(&self, x: char) -> bool {
        self.contains(&x)
    }
}

impl CharPredicate for str {
    fn check(&self, x: char) -> bool {
        self.contains(x)
    }
}

impl<'a, P: CharPredicate + ?Sized> CharPredicate for &'a P {
    fn check(&self, x: char) -> bool {
        (*self).check(x)
    }
}

/// Negation of a character predicate.
#[repr(transparent)]
pub struct NotPred<P: CharPredicate + Sized>(pub P);

impl<P: CharPredicate> CharPredicate for NotPred<P> {
    #[inline]
    fn check(&self, x: char) -> bool {
        !self.0.check(x)
    }
}

#[allow(unused_macros)]
macro_rules! not {
    ($p: expr) => {
        $crate::char::NotPred($p)
    };
}

/// Logical or of 2 character predicates.
pub struct OrPred<P: CharPredicate, Q: CharPredicate>(pub P, pub Q);

impl<P: CharPredicate, Q: CharPredicate> CharPredicate for OrPred<P, Q> {
    #[inline]
    fn check(&self, x: char) -> bool {
        self.0.check(x) || self.1.check(x)
    }
}

#[allow(unused_macros)]
macro_rules! any {
    ($p: expr) => { $p };
    ($p: expr, $($ps: expr),+) => {
        $crate::char::OrPred($p, any!($($ps),+))
    }
}

/// Logical and of 2 character predicates.
pub struct AndPred<P: CharPredicate, Q: CharPredicate>(pub P, pub Q);

impl<P: CharPredicate, Q: CharPredicate> CharPredicate for AndPred<P, Q> {
    #[inline]
    fn check(&self, x: char) -> bool {
        self.0.check(x) && self.1.check(x)
    }
}

#[allow(unused_macros)]
macro_rules! all {
    ($p: expr) => { $p };
    ($p: expr, $($ps: expr),+) => {
        $crate::char::AndPred($p, all!($($ps),+))
    }
}

macro_rules! alias {
    { $( $($(#[$meta: meta])* pub)? $p: ident = $e: expr);* $(;)? } => {
        $(
            $($(#[$meta])* pub)?
            struct $p;
            impl CharPredicate for $p {
                fn check(&self, x: char) -> bool { $e.check(x) }
            }
        )+
    }
}

/// A character stream, common interface for macros here.
pub trait Stream {
    /// Peek the next character without consuming it.
    fn peek(&mut self) -> Option<char>;
    /// Take the next character and consume it.
    fn next(&mut self) -> Option<char>;
    /// Match a string against the input.
    fn r#match<'a>(&mut self, s: &'a str) -> Option<&'a str> {
        for c in s.chars() {
            match self.next() {
                Some(x) if x == c => (),
                _ => return None,
            }
        }
        Some(s)
    }
    /// Pop many characters until the predicate fails.
    fn span<T>(&mut self, mut f: impl FnMut(char) -> bool,
               init: T, mut join: impl FnMut(&mut T, char)) -> T {
        let mut res = init;
        while let Some(x) = self.peek() {
            if !f(x) { break; }
            join(&mut res, x);
            self.next();
        }
        res
    }
    /// Pop many characters until the predicate fails, collect them into a `Vec`.
    fn span_collect(&mut self, f: impl FnMut(char) -> bool) -> Vec<char> {
        self.span(f, Vec::new(), Vec::push)
    }
    /// Pop many characters until the predicate fails, collect them into a `String`.
    fn span_collect_string(&mut self, f: impl FnMut(char) -> bool) -> String {
        self.span(f, String::new(), String::push)
    }
    /// Pop many characters until the predicate fails, ignore the characters.
    fn span_(&mut self, f: impl FnMut(char) -> bool) {
        self.span(f, (), |_, _| ())
    }
}

macro_rules! alt {
    ($lexer: expr) => { trace!(scanner, "alt: failed"); };
    ($lexer: expr, $f: expr $(, $($rest: tt)+)?) => {
        trace!(scanner, "alt: try parsing {}", stringify!($f));
        {
            let res = $lexer.anchored($f);
            let just = $crate::utils::Maybe::is_just(&res);
            if let Ok(val) = $crate::utils::Either::into_result(res) {
                if just {
                    trace!(scanner, "ok: {}: {:?}", stringify!($f), val);
                } else {
                    trace!(scanner, "fail fast: {}", stringify!($f));
                }
                return $crate::utils::Either::right(
                    std::convert::From::from(val));
            }
        }
        trace!(scanner, "failed: {}", stringify!($f));
        alt!($lexer $(, $($rest)+)?);
    }
}

macro_rules! simple_alt {
    ($lexer: expr $(, $($rest: tt)+)?) => {
        (|| {
            alt!($lexer, $($($rest)+)?);
            None
        })()
    }
}

macro_rules! choice {
    ($res: expr; $($rest: tt)+) => {
        |scanner| {
            analyse!(scanner, $($rest)+);
            $crate::utils::Maybe::just($res)
        }
    };
    ($($rest: tt)+) => { choice!((); $($rest)+) }
}

#[allow(unused_macros)]
macro_rules! seq {
    ($s: expr => $res: expr) => {
        |scanner| scanner.r#match($s).map(|_| $res)
    };
    ($s: expr) => {
        |scanner| scanner.r#match($s)
    }
}

macro_rules! analyse {
    ($lexer: expr) => {};
    ($lexer: expr, $x: ident : {$e: expr} {$cons: expr} * $predicate: expr $(, $($rest: tt)+)?) => {
        check!(collect($e, $cons) many, $lexer, $x, $predicate);
        analyse!($lexer $(, $($rest)+)?);
    };
    ($lexer: expr, $x: ident : {$e: expr} {$cons: expr} + $predicate: expr $(, $($rest: tt)+)?) => {
        check!(collect($e, $cons) some, $lexer, $x, $predicate);
        analyse!($lexer $(, $($rest)+)?);
    };
    ($lexer: expr, $x: ident : * $predicate: expr $(, $($rest: tt)+)?) => {
        check!(many, $lexer, $x, $predicate);
        analyse!($lexer $(, $($rest)+)?);
    };
    ($lexer: expr, $x: ident : + $predicate: expr $(, $($rest: tt)+)?) => {
        check!(some, $lexer, $x, $predicate);
        analyse!($lexer $(, $($rest)+)?);
    };
    ($lexer: expr, $x: ident : $predicate: expr $(, $($rest: tt)+)?) => {
        check!(once, $lexer, $x, $predicate);
        analyse!($lexer $(, $($rest)+)?);
    };
    ($lexer: expr, * $predicate: expr $(, $($rest: tt)+)?) => {
        check!(many, $lexer, drop __x, $predicate);
        analyse!($lexer $(, $($rest)+)?);
    };
    ($lexer: expr, + $predicate: expr $(, $($rest: tt)+)?) => {
        check!(some, $lexer, drop __x, $predicate);
        analyse!($lexer $(, $($rest)+)?);
    };
    ($lexer: expr, $predicate: expr $(, $($rest: tt)+)?) => {
        check!(once, $lexer, drop __x, $predicate);
        analyse!($lexer $(, $($rest)+)?);
    }
}

macro_rules! check {
    (once, $lexer: expr, $x: ident, $predicate: expr) => {
        let $x = $lexer.next()?;
        if !$predicate.check($x) {
            trace!(scanner, "analyse({:?}): checking {} ... failed", $x, stringify!($predicate));
            return None;
        }
        trace!(scanner, "analyse({:?}): checking {} ... ok", $x, stringify!($predicate));
        let $x = $x; // retain unused variable warnings
    };
    (once, $lexer: expr, drop $x: ident, $predicate: expr) => {
        check!(once, $lexer, $x, $predicate);
        let $x = (); // effectively drop $x
    };
    (many, $lexer: expr, $x: ident, $predicate: expr) => {
        let $x = $lexer.span_collect(|$x| $predicate.check($x));
        trace!(scanner, "analyse: checking *{} ... ok", stringify!($predicate));
    };
    (many, $lexer: expr, drop $x: ident, $predicate: expr) => {
        $lexer.span_(|$x| $predicate.check($x));
        trace!(scanner, "analyse: checking *{} ... ok", stringify!($predicate));
    };
    (some, $lexer: expr, $x: ident, $predicate: expr) => {
        let $x = $lexer.span_collect(|$x| $predicate.check($x));
        if $x.len() == 0 {
            trace!(scanner, "analyse: checking +{} ... failed", stringify!($predicate));
            return None;
        }
        trace!(scanner, "analyse: checking +{} ... ok", stringify!($predicate));
        let $x = $x; // retain unused variable warnings
    };
    (some, $lexer: expr, drop $x: ident, $predicate: expr) => {
        trace!(scanner, "analyse: checking +{0} as {0}, *{0} ...", stringify!($predicate));
        check!(once, $lexer: expr, drop $x, $predicate);
        check!(many, $lexer: expr, drop $x, $predicate);
    };
    (collect($e: expr, $cons: expr) many, $lexer: expr, $x: ident, $predicate: expr) => {
        let $x = $lexer.span(|$x| $predicate.check($x), $e, $cons);
        trace!(scanner, "analyse: checking {{{}}} {{{}}} *{} ... ok",
               stringify!($e), stringify!($cons), stringify!($predicate));
    };
    (collect($e: expr, $cons: expr) some, $lexer: expr, $x: ident, $predicate: expr) => {
        let $x = $lexer.span(|$x| $predicate.check($x), $e, $cons);
        if $x.len() == 0 {
            trace!(scanner, "analyse: checking {{{}}} {{{}}} *{} ... failed",
                   stringify!($e), stringify!($cons), stringify!($predicate));
            return None;
        }
        trace!(scanner, "analyse: checking {{{}}} {{{}}} *{} ... ok",
               stringify!($e), stringify!($cons), stringify!($predicate));
        let $x = $x; // retain unused variable warnings
    }
}

#[cfg(test)]
mod tests {
    use super::{Unicode, Ascii, CharPredicate, Stream};
    use crate::scanner::Scanner;

    #[test]
    fn test_syntax() {
        #[allow(dead_code)]
        #[allow(unused_variables)]
        fn parse<I: std::io::Read>(scanner: &mut Scanner<I>) -> Option<()> {
            analyse!(scanner);
            analyse!(scanner, x: Ascii::Any);
            analyse!(scanner, x: +Unicode::Alpha, '\n');
            analyse!(scanner, x: *any!(Unicode::Alpha, Ascii::Digit));
            analyse!(scanner, x: "aeiou");
            Some(())
        }
    }
}
