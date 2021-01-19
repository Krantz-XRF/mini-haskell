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

use clap::{Arg, App, SubCommand};

use std::fs::File;
use std::path::Path;
use mini_haskell::scanner::layout::{
    RawLexemeIterator,
    FatLexemeIterator,
    EnrichedLexemeIterator,
    AugmentedLexemeIterator,
    EnrichedLexeme,
};

fn print_lexemes(it: impl Iterator<Item=impl std::fmt::Display>) {
    for x in it { println!("{}", x) }
}

fn main() {
    let input_file = Arg::with_name("INPUT")
        .help("Haskell source file to process")
        .required(true)
        .index(1);
    let matches = App::new("mini-haskell")
        .version(concat!(env!("CARGO_PKG_VERSION")))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .subcommand(SubCommand::with_name("compile")
            .about("Compile the Haskell source file")
            .arg(input_file.clone()))
        .subcommand(SubCommand::with_name("lex")
            .about("Get lexeme stream from the lexer")
            .arg(Arg::with_name("flavour")
                .short("f")
                .long("flavour")
                .help("Select a flavour of lexer output")
                .value_name("FLAVOUR")
                .takes_value(true)
                .possible_values(&["raw", "fat", "enriched", "augmented"])
                .default_value("raw"))
            .arg(input_file))
        .get_matches();
    if let Some(sub_matches) = matches.subcommand_matches("lex") {
        let path = sub_matches.value_of("INPUT").unwrap();
        let file = File::open(Path::new(path)).unwrap_or_else(|err| {
            eprintln!("cannot open file '{}': {}", path, err);
            std::process::exit(1)
        });
        match sub_matches.value_of("flavour").unwrap() {
            "raw" => print_lexemes(RawLexemeIterator::new(file)),
            "fat" => print_lexemes(FatLexemeIterator::new(file).map(EnrichedLexeme::from)),
            "enriched" => print_lexemes(EnrichedLexemeIterator::new(file)),
            "augmented" => print_lexemes(AugmentedLexemeIterator::new(file)),
            _ => unreachable!(),
        }
    } else if let Some(_sub_matches) = matches.subcommand_matches("compile") {
        eprintln!("compile not yet supported.");
        std::process::exit(1)
    }
}
