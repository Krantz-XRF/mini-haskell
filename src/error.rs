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

//! error reporting for the mini-Haskell compiler.

use crate::lexeme::LexemeType;
use crate::scanner::{LexError, Location, Range};

/// An exhaustive list of compiler errors.
#[derive(Debug)]
pub enum Error {
    /// An invalid UTF-8 sequence.
    InvalidUTF8(Vec<u8>),
    /// A failure during the input process.
    InputFailure(std::io::Error),
    /// A Unicode character not accepted by the Haskell language.
    InvalidChar(char),
    /// An error during the tokenization process.
    InvalidToken(LexError),
    /// A lexeme ended prematurely, e.g. EOF in a block comment.
    IncompleteLexeme(LexemeType),
}

/// A diagnostic message (body).
#[derive(Debug)]
pub enum DiagnosticMessage {
    /// Critical errors.
    Error(Error),
}

/// A diagnostic, with a source location, and an optional source range.
#[derive(Debug)]
pub struct Diagnostic {
    location: Location,
    range: Option<Range>,
    message: DiagnosticMessage,
}

impl Diagnostic {
    /// Create a new diagnostics.
    pub fn new(location: Location, message: DiagnosticMessage) -> Diagnostic {
        Diagnostic { location, message, range: None }
    }

    /// Add a source range to the report.
    pub fn within_range(self, range: Range) -> Self {
        Self { range: Some(range), ..self }
    }

    /// Add a source range from a `[begin, end)` pair to the report.
    pub fn within(self, begin: Location, end: Location) -> Self {
        Self { range: Some(Range { begin, end }), ..self }
    }

    /// Report to the diagnostics engine.
    pub fn report(self, engine: &mut DiagnosticsEngine) {
        engine.push(self)
    }
}

/// The diagnostics engine.
pub type DiagnosticsEngine = Vec<Diagnostic>;
