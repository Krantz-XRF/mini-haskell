/*
 * mini-haskell: light-weight Haskell for fun
 * Copyright (C) 2021  Xie Ruifeng
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

//! `rlex` abstract syntax types.

pub mod char_class;
pub mod op;

pub use char_class::UnicodeCharClass;
pub use op::RegOp;

use std::convert::{TryFrom, TryInto};
use syn::{LitChar, LitStr};
use crate::syntax::*;
use crate::ast::char_class::UnicodeCharRange;
use syn::__private::fmt::Display;
use syn::__private::Formatter;
use crate::ast::op::Pretty;

type Result<T> = std::result::Result<T, syn::Error>;

/// `RegEx a = fix (RegOp a)`.
pub struct RegEx<A>(RegOp<A, RegEx<A>>);

impl<A> RegEx<A> {
    /// `fmap` for functors.
    pub fn fmap<B>(self, f: impl Fn(A) -> B) -> RegEx<B> {
        RegEx(self.0.bimap(&f, |r| r.fmap(&f)))
    }

    /// `fold` for fixed points.
    pub fn fold<B>(self, f: impl Fn(RegOp<A, B>) -> B) -> B {
        f(self.0.bimap(std::convert::identity, |r| r.fold(&f)))
    }

    /// `unfold` for fixed points.
    pub fn unfold<B>(x: B, f: impl Fn(B) -> RegOp<A, B>) -> RegEx<A> {
        RegEx(f(x).bimap(std::convert::identity, |r| RegEx::unfold(r, &f)))
    }
}

impl<A: Display> Pretty for RegEx<A> {
    type Context = usize;
    fn pretty_fmt(&self, f: &mut Formatter<'_>, n: usize) -> std::fmt::Result {
        self.0.pretty_fmt(f, n)
    }
}

impl<A: Display> Display for RegEx<A> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.pretty_fmt(f, 0)
    }
}

fn unwrap_first<It: IntoIterator>(c: It) -> It::Item {
    c.into_iter().next().unwrap()
}

fn collect_vec<T>(xs: T) -> Result<Vec<RegEx<UnicodeCharClass>>>
    where T: IntoIterator, T::Item: TryInto<RegEx<UnicodeCharClass>, Error=syn::Error> {
    xs.into_iter()
        .map(TryInto::try_into)
        .fold(Ok(Vec::new()), |r, x| match (r, x) {
            (Err(e), Ok(_)) => Err(e),
            (Ok(_), Err(e)) => Err(e),
            (Ok(mut r), Ok(x)) => {
                r.push(x);
                Ok(r)
            }
            (Err(mut e1), Err(e2)) => {
                e1.combine(e2);
                Err(e1)
            }
        })
}

impl TryFrom<Expr> for RegEx<UnicodeCharClass> {
    type Error = syn::Error;
    fn try_from(e: Expr) -> Result<Self> {
        if e.variants.is_empty() {
            Ok(RegEx(RegOp::Atom(UnicodeCharClass::empty())))
        } else if e.variants.len() == 1 {
            unwrap_first(e.variants).try_into()
        } else {
            collect_vec(e.variants).map(|r| RegEx(RegOp::Alt(r)))
        }
    }
}

impl TryFrom<Concat> for RegEx<UnicodeCharClass> {
    type Error = syn::Error;
    fn try_from(c: Concat) -> Result<Self> {
        assert!(!c.items.is_empty(), "guaranteed by Punctuated::parse_separated_nonempty");
        if c.items.len() == 1 {
            unwrap_first(c.items).try_into()
        } else {
            collect_vec(c.items).map(|r| RegEx(RegOp::Concat(r)))
        }
    }
}

impl TryFrom<Repeat> for RegEx<UnicodeCharClass> {
    type Error = syn::Error;
    fn try_from(rep: Repeat) -> Result<Self> {
        match rep {
            Repeat::Once(x) => x.try_into(),
            Repeat::Optional(x, _) => Ok(RegEx(RegOp::Optional(Box::new(x.try_into()?)))),
            Repeat::Some(x, _) => Ok(RegEx(RegOp::Some(Box::new(x.try_into()?)))),
            Repeat::Many(x, _) => Ok(RegEx(RegOp::Optional(Box::new(
                RegEx(RegOp::Some(Box::new(
                    x.try_into()?
                )))
            )))),
        }
    }
}

impl TryFrom<Atom> for RegEx<UnicodeCharClass> {
    type Error = syn::Error;
    fn try_from(a: Atom) -> Result<Self> {
        match a {
            Atom::Char(c) => Ok(RegEx(RegOp::Atom(c.into()))),
            Atom::String(s) => Ok(s.into()),
            Atom::Range(r) => Ok(RegEx(RegOp::Atom(r.into()))),
            Atom::Class(c) => Ok(RegEx(RegOp::Atom(c.try_into()?))),
            Atom::Paren { expr, .. } => expr.try_into(),
        }
    }
}

impl From<LitChar> for UnicodeCharClass {
    fn from(c: LitChar) -> Self {
        UnicodeCharClass::from(c.value())
    }
}

impl From<LitStr> for RegEx<UnicodeCharClass> {
    fn from(s: LitStr) -> Self {
        let xs = s.value().chars()
            .map(UnicodeCharClass::from)
            .map(RegOp::Atom)
            .map(RegEx)
            .collect::<Vec<_>>();
        if xs.len() == 1 {
            unwrap_first(xs)
        } else {
            RegEx(RegOp::Concat(xs))
        }
    }
}

impl TryFrom<CharClass> for UnicodeCharClass {
    type Error = syn::Error;
    fn try_from(cls: CharClass) -> Result<Self> {
        fn fst<T: Copy, U>(x: &(T, U)) -> T { x.0 }
        fn make_char_class(by_names: &[(&'static str, &'static [(u32, u32)])], name: &str)
                           -> UnicodeCharClass {
            let i = by_names.binary_search_by_key(&name, fst).unwrap();
            UnicodeCharClass::from_sorted(
                by_names[i].1.iter().copied()
                    .map(|(l, r)| UnicodeCharRange::from_raw(l, r + 1))
                    .collect())
        }
        use crate::unicode_tables::{
            GEN_CATS,
            property_names::PROPERTY_NAMES,
            property_bool::BY_NAME as PROPERTY_BOOL,
            general_category::BY_NAME as GENERAL_CATEGORY,
        };
        let mut cls_str = cls.class_name.to_string();
        ucd_util::symbolic_name_normalize(&mut cls_str);
        if let Some(prop) = ucd_util::canonical_property_name(
            PROPERTY_NAMES, &cls_str) {
            Ok(make_char_class(PROPERTY_BOOL, prop))
        } else if let Some(cat) = ucd_util::canonical_property_value(
            GEN_CATS, &cls_str) {
            Ok(make_char_class(GENERAL_CATEGORY, cat))
        } else {
            Err(syn::Error::new(
                cls.class_name.span(),
                format!("'{}' is not a valid Unicode property, nor is it a valid value for \
                    property General_Category, even after normalization specified by UAX44-LM3 \
                    (where it becomes '{}').",
                        cls.class_name, cls_str),
            ))
        }
    }
}

impl From<CharRange> for UnicodeCharClass {
    fn from(r: CharRange) -> Self {
        UnicodeCharClass::from(
            UnicodeCharRange::new(
                r.begin.value(), r.end.value()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_quote;

    #[test]
    fn test_ast() {
        let exprs: Vec<Expr> = vec![
            parse_quote!($NonSense $WhiteSpace $AlsoNonSense),
            parse_quote!($WhiteSpace),
            parse_quote!('a'..'z'),
            parse_quote!('a'?),
            parse_quote!('0'..'9' | 'a'..'f' | 'A'..'F'),
            parse_quote!("Bonjour" ','? "le"* "monde"),
        ];
        let expected = [
            Err([
                "'NonSense' is not a valid Unicode property, nor is it a valid value for \
                property General_Category, even after normalization specified by UAX44-LM3 \
                (where it becomes 'nonsense').",
                "'AlsoNonSense' is not a valid Unicode property, nor is it a valid value for \
                property General_Category, even after normalization specified by UAX44-LM3 \
                (where it becomes 'alsononsense').",
            ]),
            Ok("[[\t-\r] \u{85}\u{a0}\u{1680}[\u{2000}-\u{200a}][\u{2028}-\u{2029}]\u{202f}\u{205f}\u{3000}]"),
            Ok("[a-z]"),
            Ok("[a]?"),
            Ok("[0-9] | [a-f] | [A-F]"),
            Ok("[B] [o] [n] [j] [o] [u] [r] [,]? ([l] [e])+? [m] [o] [n] [d] [e]"),
        ];
        for (expr, ans) in exprs.into_iter().zip(expected.iter()) {
            let expr: Result<RegEx<UnicodeCharClass>> = expr.try_into();
            match expr {
                Ok(expr) =>
                    assert_eq!(expr.to_string(), *ans.unwrap()),
                Err(err) => {
                    let ans = ans.unwrap_err();
                    let mut err_ans = ans.iter();
                    for msg in err {
                        assert_eq!(msg.to_string(), *err_ans.next().unwrap());
                    }
                }
            }
        }
    }
}
