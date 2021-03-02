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

use syn::{
    Token,
    parenthesized, braced, token,
    Ident, LitChar, LitStr, Visibility,
    punctuated::Punctuated, parse::{Parse, Result, ParseBuffer},
};
use quote::{quote, ToTokens, TokenStreamExt};
use proc_macro2::TokenStream;

/// for IntelliJ Rust (intellisense):
/// macro 'Token!' is `syn::Token`, but incorrectly resolves to `syn::token::Token`
#[allow(unused_imports)]
use syn::token::Token;

pub struct Rule {
    pub vis: Visibility,
    pub name: Ident,
    _equal_sign: Token![=],
    pub body: Expr,
}

impl Parse for Rule {
    fn parse<'a>(input: &'a ParseBuffer<'a>) -> Result<Self> {
        Ok(Rule {
            vis: input.parse()?,
            name: input.parse()?,
            _equal_sign: input.parse()?,
            body: input.parse()?,
        })
    }
}

impl ToTokens for Rule {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Rule { vis, name, .. } = self;
        tokens.append_all(quote! { #vis #name })
    }
}

pub type Expr = Alt;

pub struct Alt {
    pub variants: Punctuated<Concat, Token![|]>,
}

impl Parse for Alt {
    fn parse<'a>(input: &'a ParseBuffer<'a>) -> Result<Self> {
        type P = Punctuated<Concat, Token![|]>;
        Ok(Alt { variants: P::parse_separated_nonempty(input)? })
    }
}

pub struct Concat {
    pub items: Vec<Repeat>,
}

impl Parse for Concat {
    fn parse<'a>(input: &'a ParseBuffer<'a>) -> Result<Self> {
        let mut items = Vec::new();
        while input.peek(LitChar) ||
            input.peek(LitStr) ||
            input.peek(Token![$]) ||
            input.peek(token::Paren) {
            items.push(input.parse()?);
        }
        Ok(Concat { items })
    }
}

pub enum Repeat {
    Once(Atom),
    Many(Atom, Token![*]),
    Some(Atom, Token![+]),
    Optional(Atom, Token![?]),
}

impl Parse for Repeat {
    fn parse<'a>(input: &'a ParseBuffer<'a>) -> Result<Self> {
        let a = input.parse()?;
        Ok(if input.peek(Token![*]) {
            Repeat::Many(a, input.parse()?)
        } else if input.peek(Token![+]) {
            Repeat::Some(a, input.parse()?)
        } else if input.peek(Token![?]) {
            Repeat::Optional(a, input.parse()?)
        } else {
            Repeat::Once(a)
        })
    }
}

pub enum Atom {
    Char(LitChar),
    String(LitStr),
    Range(CharRange),
    Class(CharClass),
    Paren {
        _parenthesis: token::Paren,
        expr: Expr,
    },
}

impl Parse for Atom {
    fn parse<'a>(input: &'a ParseBuffer<'a>) -> Result<Self> {
        #![allow(clippy::eval_order_dependence)]
        let lookahead = input.lookahead1();
        if lookahead.peek(LitChar) {
            if input.peek2(Token![..]) {
                input.parse().map(Atom::Range)
            } else {
                input.parse().map(Atom::Char)
            }
        } else if lookahead.peek(LitStr) {
            input.parse().map(Atom::String)
        } else if lookahead.peek(Token![$]) {
            input.parse().map(Atom::Class)
        } else if lookahead.peek(token::Paren) {
            let content;
            Ok(Atom::Paren {
                _parenthesis: parenthesized!(content in input),
                expr: content.parse()?,
            })
        } else {
            Err(lookahead.error())
        }
    }
}

pub struct CharClass {
    _class_sign: Token![$],
    pub class_name: Ident,
}

impl Parse for CharClass {
    fn parse<'a>(input: &'a ParseBuffer<'a>) -> Result<Self> {
        Ok(CharClass {
            _class_sign: input.parse()?,
            class_name: input.parse()?,
        })
    }
}

pub struct CharRange {
    pub begin: LitChar,
    _range_sign: Token![..],
    pub end: LitChar,
}

impl Parse for CharRange {
    fn parse<'a>(input: &'a ParseBuffer<'a>) -> Result<Self> {
        Ok(CharRange {
            begin: input.parse()?,
            _range_sign: input.parse()?,
            end: input.parse()?,
        })
    }
}

pub enum ConditionTrans {
    Simple(Ident),
    Trans {
        begin: Ident,
        _trans_sign: Token![->],
        end: Ident,
    },
}

impl Parse for ConditionTrans {
    fn parse<'a>(input: &'a ParseBuffer<'a>) -> Result<Self> {
        if input.peek2(Token![->]) {
            Ok(ConditionTrans::Trans {
                begin: input.parse()?,
                _trans_sign: input.parse()?,
                end: input.parse()?,
            })
        } else {
            input.parse().map(ConditionTrans::Simple)
        }
    }
}

pub struct StartCondition {
    _angle_left: Token![<],
    pub condition: Punctuated<ConditionTrans, Token![,]>,
    _angle_right: Token![>],
}

impl Parse for StartCondition {
    fn parse<'a>(input: &'a ParseBuffer<'a>) -> Result<Self> {
        type P = Punctuated<ConditionTrans, Token![,]>;
        Ok(StartCondition {
            _angle_left: input.parse()?,
            condition: P::parse_separated_nonempty(input)?,
            _angle_right: input.parse()?,
        })
    }
}

pub struct Group {
    _body_brace: token::Brace,
    pub body: Punctuated<Rule, Token![;]>,
}

impl Parse for Group {
    fn parse<'a>(input: &'a ParseBuffer<'a>) -> Result<Self> {
        #![allow(clippy::eval_order_dependence)]
        let content;
        Ok(Group {
            _body_brace: braced!(content in input),
            body: content.parse_terminated(Rule::parse)?,
        })
    }
}

impl ToTokens for Group {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.body.iter().for_each(|r| r.to_tokens(tokens))
    }
}

pub enum RuleBlock {
    Group(Group),
    Single(Rule, Token![;]),
}

impl Parse for RuleBlock {
    fn parse<'a>(input: &'a ParseBuffer<'a>) -> Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(token::Brace) {
            input.parse().map(RuleBlock::Group)
        } else {
            Ok(RuleBlock::Single(input.parse()?, input.parse()?))
        }
    }
}

impl ToTokens for RuleBlock {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            RuleBlock::Group(g) => g.to_tokens(tokens),
            RuleBlock::Single(r, _) => r.to_tokens(tokens),
        }
    }
}

pub struct WithCondition {
    pub start_condition: Option<StartCondition>,
    pub body: RuleBlock,
}

impl Parse for WithCondition {
    fn parse<'a>(input: &'a ParseBuffer<'a>) -> Result<Self> {
        if input.peek(Token![<]) {
            Ok(WithCondition {
                start_condition: Some(input.parse()?),
                body: input.parse()?,
            })
        } else {
            Ok(WithCondition {
                start_condition: None,
                body: input.parse()?,
            })
        }
    }
}

impl ToTokens for WithCondition {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.body.to_tokens(tokens)
    }
}

pub struct LexemeDef {
    pub _enum_token: Token![enum],
    pub name: Ident,
    pub _body_brace: token::Brace,
    pub body: Vec<WithCondition>,
}

impl Parse for LexemeDef {
    fn parse<'a>(input: &'a ParseBuffer<'a>) -> Result<Self> {
        #![allow(clippy::eval_order_dependence)]
        let contents;
        Ok(LexemeDef {
            _enum_token: input.parse()?,
            name: input.parse()?,
            _body_brace: braced!(contents in input),
            body: {
                let mut body = Vec::new();
                while !contents.is_empty() {
                    body.push(contents.parse()?)
                }
                body
            },
        })
    }
}

impl ToTokens for LexemeDef {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let LexemeDef { _enum_token, name, body, .. } = self;
        tokens.append_all(quote! { #_enum_token #name { #(#body),* } })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_quote;

    #[test]
    #[allow(unused_variables)]
    fn test_syntax() {
        let class: CharClass = parse_quote!($WhiteSpace);
        let repeat: Repeat = parse_quote!($WhiteSpace?);
        let concat: Concat = parse_quote!($NonSense $WhiteSpace? 'a');
        let expr: Expr = parse_quote!($NonSense $WhiteSpace? 'a');
        let rule: Rule = parse_quote!(Class = $NonSense $WhiteSpace? 'a');
        let block: Group = parse_quote!({
            Class = $NonSense $WhiteSpace? 'a';
            Range = 'a'..'z';
            Optional = 'a'?;
            pub Alt = '0'..'9' | 'a'..'f' | 'A'..'F';
        });
        let cond: StartCondition = parse_quote!(<start>);
        let cond_block: WithCondition = parse_quote! {
            <start> {
                Class = $NonSense $WhiteSpace? 'a';
                Range = 'a'..'z';
                Optional = 'a'?;
                pub Alt = '0'..'9' | 'a'..'f' | 'A'..'F';
            }
        };
        let cond_block2: WithCondition = parse_quote! {
            <someOther> Test = "Bonjour" ','? "le"? "monde";
        };
        let def: LexemeDef = parse_quote! {
            enum Lexeme {
                <start, someOther> {
                    Class = $NonSense $WhiteSpace? 'a';
                    Range = 'a'..'z';
                    Optional = 'a'?;
                    pub Alt = '0'..'9' | 'a'..'f' | 'A'..'F';
                }
                Test = "Bonjour" ','? "le"* "monde";
                <start -> someOther> Test2 = "Bonjour" ','? "le"? "monde";
            }
        };
    }
}
