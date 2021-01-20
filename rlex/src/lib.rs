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

//! Procedural macros for defining lexers.

mod ast;

/// `rlex! { ... }` will generate a DFA-based lexer.
#[proc_macro]
pub fn rlex(tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = proc_macro2::TokenStream::from(tokens);
    let output = rlex_impl(input);
    proc_macro::TokenStream::from(output)
}

fn rlex_impl(_input: proc_macro2::TokenStream) -> proc_macro2::TokenStream {
    todo!()
}