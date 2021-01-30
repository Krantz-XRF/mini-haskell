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
use std::fmt::{Display, Formatter};
use std::collections::BTreeSet;
use syn::{LitChar, LitStr};

use crate::syntax::*;
use crate::ast::char_class::UnicodeCharRange;
use crate::ast::op::{Pretty, ForEach};

type Result<T> = std::result::Result<T, syn::Error>;

/// `RegEx a = fix (RegOp a)`.
pub struct RegEx<A>(RegOp<A, RegEx<A>>);

impl<A> ForEach for RegEx<A> {
    type Item = A;
    fn for_each(&self, f: &mut impl FnMut(&A)) {
        self.0.for_each(f)
    }
}

impl<A> RegEx<A> {
    pub fn fmap<B>(self, f: &impl Fn(A) -> B) -> RegEx<B> {
        RegEx(self.0.bimap(f, |r| r.fmap(f)))
    }
}

impl RegEx<UnicodeCharClass> {
    fn collect_split_points(&self, res: &mut BTreeSet<u32>) {
        res.insert(0);
        res.insert(0x10FFFF + 1);
        self.for_each(&mut |cls| cls.iter()
            .flat_map(UnicodeCharRange::end_points)
            .for_each(|x| { res.insert(x); }));
    }

    pub fn classify_chars_with(self, split_points: &Vec<u32>) -> RegEx<Vec<u32>> {
        self.fmap(&|cls| {
            let mut res = BTreeSet::new();
            for UnicodeCharRange { begin, end } in cls {
                let l = split_points.binary_search(&begin).unwrap();
                let r = split_points.binary_search(&end).unwrap();
                for k in l..r {
                    res.insert(k as u32);
                }
            }
            res.into_iter().collect()
        })
    }

    pub fn classify_chars(self) -> (Vec<u32>, RegEx<Vec<u32>>) {
        let mut split_points = BTreeSet::new();
        self.collect_split_points(&mut split_points);
        let split_points = split_points.into_iter().collect::<Vec<_>>();
        let regex = self.classify_chars_with(&split_points);
        (split_points, regex)
    }
}

impl<A: Display> Pretty for Vec<A> {
    type Context = ();
    fn pretty_fmt(&self, f: &mut Formatter<'_>, _: ()) -> std::fmt::Result {
        write!(f, "{{")?;
        let mut xs = self.iter();
        if let Some(x0) = xs.next() {
            write!(f, "{}", x0)?;
            for x in xs {
                write!(f, ", {}", x)?;
            }
        }
        write!(f, "}}")
    }
}

impl<A: Pretty<Context=()>> Pretty for RegEx<A> {
    type Context = usize;
    fn pretty_fmt(&self, f: &mut Formatter<'_>, n: usize) -> std::fmt::Result {
        self.0.pretty_fmt(f, n)
    }
}

impl<A: Pretty<Context=()>> Display for RegEx<A> {
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

    #[test]
    fn test_classify_chars() {
        let expr: Expr = parse_quote!('0'..'9' | 'a'..'f' | 'A'..'F');
        let expr: RegEx<_> = expr.try_into().unwrap();
        let (chars, expr) = expr.classify_chars();
        assert_eq!(chars, vec![
            0,
            '0' as u32, '9' as u32 + 1,
            'A' as u32, 'F' as u32 + 1,
            'a' as u32, 'f' as u32 + 1,
            0x10FFFF + 1
        ]);
        assert_eq!(format!("{}", expr), "{1} | {5} | {3}");
    }
}
