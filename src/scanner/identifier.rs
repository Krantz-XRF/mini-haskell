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

//! identifiers: see "Haskell 2010 Report: 2.4 Identifiers and Operators".

use super::{Scanner, Result};
use crate::char::{CharPredicate, Stream, Unicode, Ascii};
use crate::lexeme::{RId, ROp, Lexeme, QName, ModuleId};
use crate::lexeme::Lexeme::{ReservedId, ReservedOp, Identifier, Operator, QIdentifier, QOperator};
use crate::lexeme::LexemeType as T;

alias! {
    /// small    -> ascSmall | uniSmall | _
    /// ascSmall -> a | b | ... | z
    /// uniSmall -> any Unicode lowercase letter
    pub Small = any!(Ascii::Lower, Unicode::Lower, '_');

    /// large    -> ascLarge | uniLarge
    /// ascLarge -> A | B | ... | Z
    /// uniLarge -> any uppercase or titlecase Unicode letter
    pub Large = any!(Ascii::Upper, Unicode::Upper);

    /// digit    -> ascDigit | uniDigit
    /// ascDigit -> 0 | 1 | ... | 9
    /// uniDigit -> any Unicode decimal digit
    pub Digit = any!(Ascii::Digit, Unicode::Digit);

    /// symbol    -> ascSymbol | uniSymbol<special | _ | " | '>
    /// ascSymbol -> ! | # | $ | % | & | * | + | . | / | < | = | > | ? | @
    ///            | \ | ^ | | | - | ~ | :
    /// uniSymbol -> any Unicode symbol or punctuation
    /// special   -> ( | ) | , | ; | [ | ] | ` | { | }
    pub Symbol = any!(r"!#$%&*+./<=>?@\^|-~:",
                      all!(any!(Unicode::Symbol, Unicode::Punct),
                           not!(r#"(),;[]```{}_"'"#)));
}

impl<I: std::io::Read> Scanner<I> {
    /// Identifiers or operators.
    pub fn id_or_sym(&mut self) -> Result<Lexeme> {
        alt!(self, method!(q_var_id_or_q_sym),
                   method!(q_con_id),
                   method!(con_sym_or_reserved_op),
                   method!(var_sym_or_reserved_op),
                   method!(var_id_or_reserved_id));
        todo!()
    }

    fn con_id(&mut self) -> Option<String> {
        // conid    -> large { small | large | digit | ' }
        analyse!(self, c: Large, name: {c.to_string()}{String::push} *any!(Small, Large, Digit, '\''));
        Some(name)
    }

    fn var_id_or_reserved_id(&mut self) -> Option<Lexeme> {
        // varid      -> (small { small | large | digit | ' })<reservedid>
        analyse!(self, c: Small, name: {c.to_string()}{String::push} *any!(Small, Large, Digit, '\''));
        // reservedid -> case | class | data | default | deriving | do | else
        //             | foreign | if | import | in | infix | infixl
        //             | infixr | instance | let | module | newtype | of
        //             | then | type | where | _
        Some(match name.as_str() {
            "case" => ReservedId(RId::Case),
            "class" => ReservedId(RId::Class),
            "data" => ReservedId(RId::Data),
            "default" => ReservedId(RId::Default),
            "deriving" => ReservedId(RId::Deriving),
            "do" => ReservedId(RId::Do),
            "else" => ReservedId(RId::Else),
            "foreign" => ReservedId(RId::Foreign),
            "if" => ReservedId(RId::If),
            "import" => ReservedId(RId::Import),
            "in" => ReservedId(RId::In),
            "infix" => ReservedId(RId::Infix),
            "infixl" => ReservedId(RId::Infixl),
            "infixr" => ReservedId(RId::Infixr),
            "instance" => ReservedId(RId::Instance),
            "let" => ReservedId(RId::Let),
            "module" => ReservedId(RId::Module),
            "newtype" => ReservedId(RId::Newtype),
            "of" => ReservedId(RId::Of),
            "then" => ReservedId(RId::Then),
            "type" => ReservedId(RId::Type),
            "where" => ReservedId(RId::Where),
            "_" => ReservedId(RId::Wildcard),
            _ => Identifier(name),
        })
    }

    fn mod_id(&mut self) -> Option<ModuleId> {
        // modid    -> { conid . } conid
        let names: Option<Vec<String>> = self.sep_by(
            method!(con_id), choice!('.'), Vec::new(), Vec::push);
        names.map(ModuleId)
    }

    fn var_sym_or_reserved_op(&mut self) -> Option<Lexeme> {
        // varsym       -> ( symbol<:> {symbol} )<reservedop | dashes>
        // reservedop   -> .. | : | :: | = | \ | | | <- | -> | @ | ~ | =>
        analyse!(self, c: all!(Symbol, not!(':')), name: {c.to_string()}{String::push} *Symbol);
        Some(match name.as_str() {
            ".." => ReservedOp(ROp::DotDot),
            "=" => ReservedOp(ROp::EqualSign),
            "\\" => ReservedOp(ROp::Backslash),
            "|" => ReservedOp(ROp::Pipe),
            "<-" => ReservedOp(ROp::LeftArrow),
            "->" => ReservedOp(ROp::RightArrow),
            "@" => ReservedOp(ROp::AtSign),
            "^" => ReservedOp(ROp::Tilde),
            "=>" => ReservedOp(ROp::DoubleRightArrow),
            _ => Operator(name),
        })
    }

    fn con_sym_or_reserved_op(&mut self) -> Option<Lexeme> {
        // consym       -> ( : {symbol} )<reservedop>
        // reservedop   -> .. | : | :: | = | \ | | | <- | -> | @ | ~ | =>
        analyse!(self, ':', name: {':'.to_string()}{String::push} *Symbol);
        Some(match name.as_str() {
            ":" => ReservedOp(ROp::Colon),
            "::" => ReservedOp(ROp::ColonColon),
            _ => Operator(name),
        })
    }

    fn q_con_id(&mut self) -> Result<Lexeme> {
        let err = self.err_expected(T::QIdentifier);
        let init = QName::new(self.con_id().ok_or(err)?);
        Option::map(
            self.many(|scanner| {
                analyse!(scanner, '.');
                scanner.con_id()
            }, init, QName::append),
            QIdentifier,
        ).ok_or(err)
    }

    fn q_var_id_or_q_sym(&mut self) -> Result<Lexeme> {
        let err = self.err_expected(T::QIdentifier);
        let module = self.mod_id().ok_or(err)?;
        Ok(match simple_alt!(self,
            method!(var_id_or_reserved_id),
            method!(var_sym_or_reserved_op),
            method!(con_sym_or_reserved_op)).ok_or(err)? {
            Identifier(name) => QIdentifier(QName { module, name }),
            Operator(name) => QOperator(QName { module, name }),
            _ => return Err(err),
        })
    }
}
