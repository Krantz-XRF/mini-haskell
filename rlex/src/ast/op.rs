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

impl<A: Display, R: Pretty<Context=usize>> Pretty for RegOp<A, R> {
    type Context = usize;
    fn pretty_fmt(&self, f: &mut Formatter<'_>, n: usize) -> std::fmt::Result {
        match self {
            RegOp::Atom(a) => write!(f, "{}", a),
            RegOp::Alt(rs) => sep_by(f, rs.iter(), (0, " | "), n),
            RegOp::Concat(rs) => sep_by(f, rs.iter(), (1, " "), n),
            RegOp::Some(r) => postfix(f, r, (2, "+"), n),
            RegOp::Optional(r) => postfix(f, r, (2, "?"), n),
        }
    }
}

impl<A: Display, R: Pretty<Context=usize>> Display for RegOp<A, R> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.pretty_fmt(f, 0)
    }
}

impl<A, R> RegOp<A, R> {
    /// `bimap` for bifunctors.
    pub fn bimap<B, S>(self, f: impl Fn(A) -> B, g: impl Fn(R) -> S) -> RegOp<B, S> {
        use RegOp::*;
        match self {
            Atom(x) => Atom(f(x)),
            Alt(xs) => Alt(xs.into_iter().map(g).collect()),
            Concat(xs) => Concat(xs.into_iter().map(g).collect()),
            Some(x) => Some(Box::new(g(*x))),
            Optional(x) => Optional(Box::new(g(*x))),
        }
    }
}
