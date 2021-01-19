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

//! numeric literals: see "Haskell 2010 Report: 2.5 Numeric Literals".

use super::{Scanner, Result, basic::*};

use num_bigint::BigInt;
use num_traits::{identities::Zero, ToPrimitive, Signed};

use crate::utils::char::{CharPredicate, Stream};
use crate::lexeme::{Rational, Lexeme};
use crate::lexeme::Lexeme::{Integer, Float};
use crate::error::Diagnostic;
use crate::error::DiagnosticMessage::Error;
use crate::error::Error::FloatOutOfBound;
use crate::scanner::Location;

/// Maximum allowed exponent in a floating number.
pub const MAXIMUM_EXPONENT: i64 = 4096;

impl<I: std::io::Read> Scanner<I> {
    /// Numeric literals: integers or floats.
    pub fn numeric_literal(&mut self) -> Result<Lexeme> {
        alt!(self, Self::float, Self::integer);
        Self::keep_trying()
    }

    pub(super) fn app_int(base: u32) -> impl Fn(&mut BigInt, char) {
        move |r, x| {
            *r *= base;
            *r += x.to_digit(base).unwrap()
        }
    }

    fn decimal_cont(&mut self, x: BigInt) -> Option<(usize, BigInt)> {
        // decimal      -> digit{digit}
        let cont = |(n, d): &mut (usize, BigInt), c: char| {
            Self::app_int(10)(d, c);
            *n += 1
        };
        analyse!(self, d: {(0, x)}{cont} +Digit);
        Some(d)
    }

    fn decimal(&mut self) -> Option<BigInt> {
        self.decimal_cont(BigInt::from(0)).map(|(_, x)| x)
    }

    fn integer(&mut self) -> Option<Lexeme> {
        // octal        -> octit{octit}
        // hexadecimal  -> hexit{hexit}
        // integer      -> decimal
        //               | 0o octal | 0O octal
        //               | 0x hexadecimal | 0X hexadecimal
        simple_alt!(self,
            choice!(d; '0', "oO", d: {BigInt::from(0)}{Self::app_int(8)} +Octit),
            choice!(d; '0', "xX", d: {BigInt::from(0)}{Self::app_int(16)} +Hexit),
            Self::decimal).map(Integer)
    }

    fn make_float(&mut self, d: BigInt, n: usize, mut exp: BigInt,
                  start_loc: Location) -> Option<Rational> {
        exp -= n;
        Some(match exp.to_i64() {
            Some(x) if (0..=MAXIMUM_EXPONENT).contains(&x) =>
                Rational::from(d * BigInt::from(10).pow(x as u32)),
            Some(x) if (-MAXIMUM_EXPONENT..0).contains(&x) =>
                Rational::new(d, BigInt::from(10).pow((-x) as u32)),
            _ => {
                let signum = exp.signum();
                Diagnostic::new(self.location, Error(FloatOutOfBound(exp)))
                    .within(start_loc, self.location)
                    .report(&mut self.diagnostics);
                Rational::new(signum, BigInt::zero())
            }
        })
    }

    fn float1(&mut self) -> Option<Rational> {
        let start_loc = self.location;
        // float    -> decimal . decimal [exponent]
        let d = self.decimal()?;
        analyse!(self, '.');
        let (n, d) = self.decimal_cont(d)?;
        let exp = self.exponent().unwrap_or_else(BigInt::zero);
        self.make_float(d, n, exp, start_loc)
    }

    fn float2(&mut self) -> Option<Rational> {
        let start_loc = self.location;
        // float    -> decimal exponent
        let d = self.decimal()?;
        let exp = self.exponent()?;
        self.make_float(d, 0, exp, start_loc)
    }

    fn float(&mut self) -> Option<Lexeme> {
        simple_alt!(self, Self::float1, Self::float2).map(Float)
    }

    fn exponent(&mut self) -> Option<BigInt> {
        // exponent -> (e | E) [+ | -] decimal
        analyse!(self, "eE");
        let sign = self.anchored(choice!(c; c: "+-")).unwrap_or('+');
        self.decimal().map(|x| if sign == '+' { x } else { -x })
    }
}

#[cfg(test)]
mod tests {
    use num_bigint::BigInt;
    use crate::scanner::test_scanner_on;
    use crate::utils::setup_logger;
    use crate::utils::Result3::Success;
    use crate::lexeme::Lexeme::{self, Integer, Float};
    use crate::lexeme::Rational;

    #[test]
    fn test_numerics() {
        setup_logger();
        fn test(input: &str, res: Lexeme) {
            trace!(scanner, "test on {:?} ...", input);
            test_scanner_on(input, method!(numeric_literal), Success(res), None);
        }
        test("42", Integer(BigInt::from(42)));
        test("0xcd", Integer(BigInt::from(0xcd)));
        test("0o42", Integer(BigInt::from(0o42)));
        test("3.1415", Float(Rational::new(31415, 10000)));
        test("1.5e4", Float(Rational::from(BigInt::from(15000))));
        test("1.5e+3", Float(Rational::from(BigInt::from(1500))));
        test("1.5e-2", Float(Rational::new(15, 1000)));
    }
}
