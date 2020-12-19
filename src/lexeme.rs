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
    Integer(isize),
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
