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

//! Haskell lexemes.

/// Haskell `Integer`.
use std::ops::{Add, Div};
use num_bigint::BigInt;
use num_integer::Integer;

/// Haskell module identifier (`M1.M2.(...).Mn`).
#[derive(Clone, Eq, PartialEq, Debug)]
pub struct ModuleId(pub Vec<String>);

/// Haskell qualified names (`MId.name`).
#[derive(Clone, Eq, PartialEq, Debug)]
pub struct QName {
    /// the module name in a qualified identifier.
    pub module: ModuleId,
    /// the identifier name in a qualified identifier.
    pub name: String,
}

impl QName {
    /// Create a new qualified name.
    pub fn new(name: String) -> Self {
        QName { module: ModuleId(Vec::new()), name }
    }

    /// Append a name segment to a qualified name.
    pub fn append(&mut self, name: String) {
        self.module.0.push(std::mem::replace(&mut self.name, name))
    }
}

/// Haskell `Ratio`.
#[derive(Clone, Eq, PartialEq, Debug)]
pub struct Ratio<T> {
    numerator: T,
    denominator: T,
}

impl<I: Integer + for<'a> Div<&'a I, Output=I>> Ratio<I> {
    /// Create a new [`Ratio`].
    pub fn new(numerator: impl Into<I>, denominator: impl Into<I>) -> Self {
        let numerator = numerator.into();
        let denominator = denominator.into();
        let g = numerator.gcd(&denominator);
        Ratio { numerator: numerator / &g, denominator: denominator / &g }
    }
}

impl<I: Integer> From<I> for Ratio<I> {
    fn from(numerator: I) -> Self {
        Ratio { numerator, denominator: I::one() }
    }
}

impl<I: Integer> Add for Ratio<I> {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        let (g, l) = self.denominator.gcd_lcm(&rhs.denominator);
        Ratio {
            denominator: l,
            numerator: (self.numerator * rhs.denominator + rhs.numerator * self.denominator) / g,
        }
    }
}

/// Haskell `Rational`.
pub type Rational = Ratio<BigInt>;

lexemes! {
    /// Whitespaces.
    Whitespace,
    /// Identifiers.
    Identifier(String),
    /// Operators.
    Operator(String),
    /// Qualified Identifiers.
    QIdentifier(QName),
    /// Qualified Operators.
    QOperator(QName),
    /// Integers.
    Integer(BigInt),
    /// Rationals.
    Float(Rational),
    /// Character literals.
    CharLiteral(char),
    /// String literals.
    StringLiteral(String),
    /// Reserved keywords.
    ReservedId(RId),
    /// Reserved operators.
    ReservedOp(ROp),
}

/// Haskell Reserved Keywords.
#[allow(missing_docs)]
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum RId {
    Case,
    Class,
    Data,
    Default,
    Deriving,
    Do,
    Else,
    Foreign,
    If,
    Import,
    In,
    Infix,
    Infixl,
    Infixr,
    Instance,
    Let,
    Module,
    Newtype,
    Of,
    Then,
    Type,
    Where,
    Wildcard,
}

/// Haskell Reserved Operators.
#[allow(missing_docs)]
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum ROp {
    DotDot,
    Colon,
    ColonColon,
    EqualSign,
    Backslash,
    Pipe,
    LeftArrow,
    RightArrow,
    AtSign,
    Tilde,
    DoubleRightArrow,
}
