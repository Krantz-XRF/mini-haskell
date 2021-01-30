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

use std::fmt::{Display, Formatter};

/// Regex operators.
pub enum RegOp<A, R> {
    Atom(A),
    Alt(Vec<R>),
    Concat(Vec<R>),
    Some(Box<R>),
    Optional(Box<R>),
}

pub trait ForEach {
    type Item;
    fn for_each(&self, f: &mut impl FnMut(&Self::Item));
}

impl<A, R: ForEach<Item=A>> ForEach for RegOp<A, R> {
    type Item = A;
    fn for_each(&self, f: &mut impl FnMut(&A)) {
        match self {
            RegOp::Atom(a) => f(a),
            RegOp::Alt(rs) => for x in rs { x.for_each(f) }
            RegOp::Concat(rs) => for x in rs { x.for_each(f) }
            RegOp::Some(r) => r.for_each(f),
            RegOp::Optional(r) => r.for_each(f),
        }
    }
}

impl<A, R> RegOp<A, R> {
    pub fn bimap<B, S>(self, mut f: impl FnMut(A) -> B, mut g: impl FnMut(R) -> S) -> RegOp<B, S> {
        match self {
            RegOp::Atom(a) => RegOp::Atom(f(a)),
            RegOp::Alt(rs) => RegOp::Alt(rs.into_iter().map(g).collect()),
            RegOp::Concat(rs) => RegOp::Concat(rs.into_iter().map(g).collect()),
            RegOp::Some(r) => RegOp::Some(Box::new(g(*r))),
            RegOp::Optional(r) => RegOp::Optional(Box::new(g(*r))),
        }
    }
}

pub trait Pretty {
    type Context;
    fn pretty_fmt(&self, f: &mut Formatter<'_>, context: Self::Context) -> std::fmt::Result;
}

impl<P: Pretty> Pretty for &P {
    type Context = P::Context;
    fn pretty_fmt(&self, f: &mut Formatter<'_>, context: Self::Context) -> std::fmt::Result {
        P::pretty_fmt(self, f, context)
    }
}

impl<P: Pretty> Pretty for Box<P> {
    type Context = P::Context;
    fn pretty_fmt(&self, f: &mut Formatter<'_>, context: Self::Context) -> std::fmt::Result {
        P::pretty_fmt(self, f, context)
    }
}

fn sep_by<I>(f: &mut Formatter<'_>, mut xs: I,
             (k, sep): (usize, &str), n: usize) -> std::fmt::Result
    where I: Iterator, I::Item: Pretty<Context=usize> {
    if let Some(x) = xs.next() {
        if k < n { write!(f, "(")?; }
        x.pretty_fmt(f, k)?;
        for x in xs {
            write!(f, "{}", sep)?;
            x.pretty_fmt(f, k)?;
        }
        if k < n { write!(f, ")")?; }
    }
    Ok(())
}

fn postfix(f: &mut Formatter<'_>, x: impl Pretty<Context=usize>,
           (k, op): (usize, &str), n: usize) -> std::fmt::Result {
    if k < n { write!(f, "(")?; }
    x.pretty_fmt(f, k)?;
    if k < n { write!(f, ")")?; }
    write!(f, "{}", op)
}

impl<A: Pretty<Context=()>, R: Pretty<Context=usize>> Pretty for RegOp<A, R> {
    type Context = usize;
    fn pretty_fmt(&self, f: &mut Formatter<'_>, n: usize) -> std::fmt::Result {
        match self {
            RegOp::Atom(a) => a.pretty_fmt(f, ()),
            RegOp::Alt(rs) => sep_by(f, rs.iter(), (0, " | "), n),
            RegOp::Concat(rs) => sep_by(f, rs.iter(), (1, " "), n),
            RegOp::Some(r) => postfix(f, r, (2, "+"), n),
            RegOp::Optional(r) => postfix(f, r, (2, "?"), n),
        }
    }
}

impl<A: Pretty<Context=()>, R: Pretty<Context=usize>> Display for RegOp<A, R> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.pretty_fmt(f, 0)
    }
}
