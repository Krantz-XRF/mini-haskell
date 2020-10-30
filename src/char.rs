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
    fn check(&self, x: char) -> bool { *self == x }
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
    fn check(&self, x: char) -> bool { self.contains(&x) }
}

impl CharPredicate for CharRangeInclusive {
    fn check(&self, x: char) -> bool { self.contains(&x) }
}

impl CharPredicate for str {
    fn check(&self, x: char) -> bool { self.contains(x) }
}

impl<'a> CharPredicate for &'a str {
    fn check(&self, x: char) -> bool { (*self).check(x) }
}

/// Negation of a character predicate.
#[repr(transparent)]
pub struct NotPred<P: CharPredicate + Sized> (pub P);

impl<P: CharPredicate> CharPredicate for NotPred<P> {
    #[inline]
    fn check(&self, x: char) -> bool { !self.0.check(x) }
}

#[allow(unused_macros)]
macro_rules! not { ($p: expr) => { $crate::char::NotPred($p) } }

/// Logical or of 2 character predicates.
pub struct OrPred<P: CharPredicate, Q: CharPredicate>(pub P, pub Q);

impl<P: CharPredicate, Q: CharPredicate> CharPredicate for OrPred<P, Q> {
    #[inline]
    fn check(&self, x: char) -> bool { self.0.check(x) || self.1.check(x) }
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
    fn check(&self, x: char) -> bool { self.0.check(x) && self.1.check(x) }
}

#[allow(unused_macros)]
macro_rules! all {
    ($p: expr) => { $p };
    ($p: expr, $($ps: expr),+) => {
        $crate::char::AndPred($p, all!($($ps),+))
    }
}

/// Common interface for a `Option` like structure.
/// It is strictly weaker than the not yet stable `std::ops::Try` trait.
/// It is named after `Maybe`, `Just`, and `Nothing` from Haskell.
pub trait Maybe {
    /// The content type in a `Just`.
    type Content;
    /// Construct a `Just`:
    /// - `Some` for `Optional`
    /// - `Ok` for `Result`
    fn just(x: Self::Content) -> Self;
    /// Consumes the value and makes an `Optional`.
    fn into_optional(self) -> Option<Self::Content>;
    /// Is `self` a failure?
    fn is_nothing(&self) -> bool;
    /// Is `self` a success?
    fn is_just(&self) -> bool { !self.is_nothing() }
}

impl<T> Maybe for Option<T> {
    type Content = T;
    fn just(x: T) -> Self { Some(x) }
    fn into_optional(self) -> Option<Self::Content> { self }
    fn is_nothing(&self) -> bool { self.is_none() }
}

impl<T, E> Maybe for std::result::Result<T, E> {
    type Content = T;
    fn just(x: T) -> Self { Ok(x) }
    fn into_optional(self) -> Option<Self::Content> { self.ok() }
    fn is_nothing(&self) -> bool { self.is_err() }
}

macro_rules! alt {
    ($lexer: expr) => { scanner_trace!("alt: failed"); };
    ($lexer: expr, $f: expr $(, $($rest: tt)+)?) => {
        scanner_trace!("alt: try parsing {}", stringify!($f));
        if let Some(res) = $crate::char::Maybe::into_optional($lexer.anchored($f)) {
            scanner_trace!("ok: {}", stringify!($f));
            return $crate::char::Maybe::just(res);
        }
        scanner_trace!("failed: {}", stringify!($f));
        alt!($lexer $(, $($rest)+)?);
    }
}

macro_rules! simple_alt {
    ($lexer: expr $(, $($rest: tt)+)?) => {
        {
            alt!($lexer, $($($rest)+)?);
            None
        }
    }
}

macro_rules! choice {
    ($res: expr; $($rest: tt)+) => {
        |scanner| {
            analyse!(scanner, $($rest)+);
            $crate::char::Maybe::just($res)
        }
    };
    ($($rest: tt)+) => { choice!((); $($rest)+) }
}

macro_rules! analyse {
    ($lexer: expr) => {};
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
    ($count: ident, $lexer: expr, drop $__x: ident, $predicate: expr) => {
        check_impl!($count, $lexer, $__x, $predicate);
        let $__x = $__x; // effectively drop $__x
    };
    ($count: ident, $lexer: expr, $x: ident, $predicate: expr) => {
        scanner_trace!("analyse: checking {}", stringify!($predicate));
        check_impl!($count, $lexer, $x, $predicate);
    }
}

macro_rules! check_impl {
    (once, $lexer: expr, $x: ident, $predicate: expr) => {
        let $x = $lexer.next()?;
        if !$predicate.check($x) {
            scanner_trace!("analyse: checking {} ... failed", stringify!($predicate));
            return None;
        }
        scanner_trace!("analyse: checking {} ... ok", stringify!($predicate));
        let $x = $x; // retain unused variable warnings
    };
    (many, $lexer: expr, $x: ident, $predicate: expr) => {
        let $x = $crate::buffer::span_collect($lexer, |$x| { $predicate.check($x) });
        scanner_trace!("analyse: checking *{} ... ok", stringify!($predicate));
    };
    (some, $lexer: expr, $x: ident, $predicate: expr) => {
        let $x = $crate::buffer::span_collect($lexer, |$x| { $predicate.check($x) });
        if $x.len() == 0 {
            scanner_trace!("analyse: checking {} ... failed", stringify!($predicate));
            return None;
        }
        scanner_trace!("analyse: checking +{} ... ok", stringify!($predicate));
        let $x = $x; // retain unused variable warnings
    }
}

#[cfg(test)]
mod tests {
    use super::{Unicode, Ascii, CharPredicate};
    use crate::buffer::Stream;

    #[test]
    fn test_syntax() {
        #[allow(dead_code)]
        #[allow(unused_variables)]
        fn parse(scanner: &mut impl Stream) -> Option<()> {
            analyse!(scanner);
            analyse!(scanner, x: Ascii::Any);
            analyse!(scanner, x: +Unicode::Alpha, '\n');
            analyse!(scanner, x: *any!(Unicode::Alpha, Ascii::Digit));
            analyse!(scanner, x: "aeiou");
            Some(())
        }
    }
}
