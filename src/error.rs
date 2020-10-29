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

//! Error reporting for the mini-Haskell compiler.

use std::mem::ManuallyDrop;
use crate::scanner::{LexError, Location, Range};
use crate::token::LexemeType;

/// An exhaustive list of compiler errors.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum Error {
    /// A Unicode character not accepted by the Haskell language.
    InvalidChar(char),
    /// An error during the tokenization process.
    InvalidToken(LexError),
    /// A lexeme ended prematurely, e.g. EOF in a block comment.
    IncompleteLexeme(LexemeType),
}

/// A diagnostic message (body).
#[derive(Eq, PartialEq, Debug)]
pub enum DiagnosticMessage {
    /// Critical errors.
    Error(Error),
}

/// A diagnostic, with a source location, and an optional source range.
#[derive(Eq, PartialEq, Debug)]
pub struct Diagnostic {
    location: Location,
    range: Option<Range>,
    message: DiagnosticMessage,
}

/// Returned by the `DiagnosticEngine::report`, accepting an optional source range for reporting.
/// When dropped, the report is submitted to the engine.
pub struct DiagnosticReporter<'a> {
    engine: &'a mut DiagnosticEngine,
    diagnostic: ManuallyDrop<Diagnostic>,
}

impl<'a> DiagnosticReporter<'a> {
    /// Add a source range to the report.
    pub fn within_range(&mut self, range: Range) {
        self.diagnostic.range = Some(range)
    }

    /// Add a source range from a `[begin, end)` pair to the report.
    pub fn within(&mut self, begin: Location, end: Location) {
        self.diagnostic.range = Some(Range { begin, end })
    }
}

impl<'a> Drop for DiagnosticReporter<'a> {
    fn drop(&mut self) {
        self.engine.diagnostics.push(unsafe {
            ManuallyDrop::take(&mut self.diagnostic)
        })
    }
}

/// Diagnostic engine.
#[derive(Default)]
pub struct DiagnosticEngine {
    diagnostics: Vec<Diagnostic>,
}

impl DiagnosticEngine {
    /// Creates a new diagnostic engine.
    pub fn new() -> Self { Self::default() }

    /// Iterate through the (already submitted) diagnostics.
    pub fn iter(&self) -> std::slice::Iter<Diagnostic> {
        self.diagnostics.iter()
    }

    /// Report a new diagnostic, submitted immediately after the returned
    /// `DiagnosticReporter` is dropped.
    pub fn report(&mut self, location: Location, msg: DiagnosticMessage) -> DiagnosticReporter {
        DiagnosticReporter {
            engine: self,
            diagnostic: ManuallyDrop::new(Diagnostic {
                location,
                range: None,
                message: msg,
            }),
        }
    }
}
