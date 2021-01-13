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
use std::fmt::{Formatter, Debug, Display};
use num_bigint::BigInt;
use num_integer::Integer;
use logos::Logos;

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

impl Display for QName {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for m_id in self.module.0.iter() {
            write!(f, "{}.", m_id)?;
        }
        write!(f, "{}", self.name)
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

impl<I: Display> Display for Ratio<I> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} % {}", self.numerator, self.denominator)
    }
}

/// Haskell `Rational`.
pub type Rational = Ratio<BigInt>;

/// Haskell lexemes types.
#[derive(Debug, Eq, PartialEq)]
#[derive(logos::Logos)]
#[logos(subpattern commentCont = r#"[a-z\p{Ll}_A-Z\p{Lu}0-9\p{Nd}(),;\[\]`{}"' \t]"#)]
#[logos(subpattern any = r#"[a-z\p{Ll}_A-Z\p{Lu}0-9\p{Nd}!#$%&*+./<=>?@\^|-~:\p{S}\p{P}(),;\[\]`{}"' \t]"#)]
#[logos(subpattern ANYnodash = r#"[a-z\p{Ll}_A-Z\p{Lu}0-9\p{Nd}!#$%&*+./<=>?@\^|~:\p{S}\p{P}(),;\[\]`{}"'\r\n\f\v\t \p{Whitespace}]"#)]
#[logos(subpattern newline = r#"\r\n|\r|\n|\f"#)]
#[logos(subpattern id = r#"[a-z\p{Ll}_A-Z\p{Lu}][a-z\p{Ll}A-Z\p{Lu}0-9\p{Nd}_']*"#)]
#[logos(subpattern varid = r#"[a-z\p{Ll}_][a-z\p{Ll}A-Z\p{Lu}0-9\p{Nd}_']*"#)]
#[logos(subpattern modid = r#"[A-Z\p{Lu}][a-z\p{Ll}A-Z\p{Lu}0-9\p{Nd}_']*"#)]
#[logos(subpattern symbol = r#"[[!#$%&*+\./<=>?@\^|-~:\p{S}\p{P}]&&[^_"'(),;\[\]`{}]]"#)]
pub enum Lexeme {
    /// Whitespaces.
    #[regex(r"(\r\n|\r|\n|\f|\v| |\t|\p{Whitespace})+")]
    Whitespace,
    /// Line comments.
    #[regex(r"---*((?&commentCont)(?&any)*)?(?&newline)")]
    Comment,
    /// Comment blocks.
    #[regex(r"\{-", ncomment)]
    NComment,
    /// Identifiers.
    #[regex(r"(?&id)", priority = 2)]
    Identifier,
    /// Operators.
    #[regex(r"(?&symbol)+")]
    Operator,
    /// Qualified Identifiers.
    #[regex(r"(?&modid)(\.(?&modid))*(\.(?&varid))?")]
    QIdentifier,
    /// Qualified Operators.
    #[regex(r"(?&modid)(\.(?&modid))*\.(?&symbol)+")]
    QOperator,
    /// Integers.
    Integer,
    /// Rationals.
    Float,
    /// Character literals.
    CharLiteral,
    /// String literals.
    StringLiteral,
    /// Reserved keywords.
    ReservedId,
    /// Reserved operators.
    ReservedOp,
    /// Special characters.
    Special,
    /// Invalid byte sequence.
    #[error]
    Invalid,
}

#[derive(Debug, logos::Logos)]
#[logos(extras = usize)]
enum NComment {
    #[regex(r"\{-+")]
    Start,
    #[regex(r"-+\}")]
    End,
    #[regex(r"[^{}-]+")]
    Content,
    #[regex(r"-+")]
    Dashes,
    #[regex(r"\{|\}")]
    Brackets,
    #[error]
    Invalid,
}

fn ncomment(lex: &mut logos::Lexer<Lexeme>) -> Option<()> {
    let mut new_lex = NComment::lexer(lex.remainder());
    new_lex.extras = 1;
    let mut result = Some(());
    while new_lex.extras > 0 {
        match new_lex.next() {
            Some(NComment::Start) => new_lex.extras += 1,
            Some(NComment::End) => new_lex.extras -= 1,
            None => {
                result = None;
                break;
            }
            _ => (),
        }
    }
    lex.bump(new_lex.span().end);
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use logos::Logos;

    fn generic_test_on(input: &str, result: Lexeme, slice: &str) {
        let mut lexer = Lexeme::lexer(input);
        assert_eq!(lexer.next(), Some(result));
        assert_eq!(lexer.slice(), slice);
    }

    fn test_on(input: &str, result: Lexeme) {
        generic_test_on(input, result, input)
    }

    #[test]
    fn test_whitespace() {
        test_on(" \r\n\n\r\t\u{C}", Lexeme::Whitespace);
        test_on("--- | test comment here\n", Lexeme::Comment);
        test_on("{- some {{-- nest -- -} block comment -}", Lexeme::NComment);
    }

    #[test]
    fn test_identifiers() {
        test_on("some'Identifier_42", Lexeme::Identifier);
        test_on("Ctor_''233'_", Lexeme::Identifier);
        test_on("Mod.SubMod.Class", Lexeme::QIdentifier);
        test_on("Mod.SubMod.Type.function", Lexeme::QIdentifier);
        test_on("+", Lexeme::Operator);
        test_on(".", Lexeme::Operator);
        test_on("F.+", Lexeme::QOperator);
        test_on("F..", Lexeme::QOperator);
        generic_test_on("F.", Lexeme::Identifier, "F");
    }
}
