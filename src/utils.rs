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

//! useful common utilities.

/// The uninhabited type `Void`.
#[derive(Copy, Clone)]
pub enum Void {}

impl Void {
    /// Consumes a `Void` value, serves as an unreachable.
    pub fn absurd(self) -> ! { match self {} }
}

/// Result type for parsing.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Result3<T, E, M> {
    /// succeed with a result
    Success(T),
    /// fail with an error, no recovery
    FailFast(E),
    /// fail without error, allowing future recovery
    RetryLater(M),
}

/// Named after `Maybe`, `Just`, and `Nothing` from Haskell.
/// Use this for success/failure semantics, since `Either` is used to model the control flow.
pub trait Maybe {
    /// The success type in a `Just`.
    type Just;
    /// Construct a success.
    fn just(x: Self::Just) -> Self;
    /// Check whether it is a success.
    fn is_just(&self) -> bool;
    /// Check whether it is a failure.
    fn is_nothing(&self) -> bool { !self.is_just() }
    /// Convert into an `Optional`.
    fn into_optional(self) -> Option<Self::Just>;
}

impl<T> Maybe for Option<T> {
    type Just = T;
    fn just(x: T) -> Self { Some(x) }
    fn is_just(&self) -> bool { self.is_some() }
    fn into_optional(self) -> Option<Self::Just> { self }
}

impl<T, E> Maybe for Result<T, E> {
    type Just = T;
    fn just(x: T) -> Self { Ok(x) }
    fn is_just(&self) -> bool { self.is_ok() }
    fn into_optional(self) -> Option<T> { self.ok() }
}

impl<T, E, M> Maybe for Result3<T, E, M> {
    type Just = T;
    fn just(x: T) -> Self { Self::Success(x) }
    fn is_just(&self) -> bool { matches!(self, Self::Success(_)) }
    fn into_optional(self) -> Option<Self::Just> {
        match self {
            Self::Success(x) => Some(x),
            _ => None,
        }
    }
}

/// The `std::ops::Try` trait is not yet stable. We roll up our own for now.
/// It is named after `Either`, `Left`, and `Right` from Haskell.
pub trait Either {
    /// The type to propagate in a `Left`.
    type Left;
    /// The type to continue with in a `Right`.
    type Right;
    /// Construct a `Left`:
    /// - `None` for `Optional`
    /// - `Err` for `Result`
    fn left(x: Self::Left) -> Self;
    /// Construct a `Right`:
    /// - `Some` for `Optional`
    /// - `Ok` for `Result`
    fn right(x: Self::Right) -> Self;
    /// Consumes the value and makes a `Result`.
    fn into_result(self) -> Result<Self::Right, Self::Left>;
}

macro_rules! unwrap {
    ($e: expr) => {
        match $crate::utils::Either::into_result($e) {
            Ok(x) => x,
            Err(e) => return $crate::utils::Either::left(e),
        }
    }
}

impl<T> Either for Option<T> {
    type Left = Option<Void>;
    type Right = T;

    fn left(_: Option<Void>) -> Self { None }
    fn right(x: T) -> Self { Some(x) }
    fn into_result(self) -> Result<T, Option<Void>> {
        match self {
            Some(x) => Ok(x),
            None => Err(None),
        }
    }
}

impl<T, E> Either for Result<T, E> {
    type Left = E;
    type Right = T;

    fn left(x: E) -> Self { Err(x) }
    fn right(x: T) -> Self { Ok(x) }
    fn into_result(self) -> Result<T, E> { self }
}

impl<T, E> From<T> for Result3<T, E, Void> {
    fn from(x: T) -> Self { Self::Success(x) }
}

impl<T, E, M> Either for Result3<T, E, M> {
    type Left = M;
    type Right = Result3<T, E, Void>;

    fn left(m: M) -> Self { Self::RetryLater(m) }

    fn right(x: Result3<T, E, Void>) -> Self {
        match x {
            Result3::Success(x) => Self::Success(x),
            Result3::FailFast(e) => Self::FailFast(e),
            Result3::RetryLater(m) => m.absurd(),
        }
    }

    fn into_result(self) -> std::result::Result<Result3<T, E, Void>, M> {
        match self {
            Self::Success(x) => Ok(Result3::Success(x)),
            Self::FailFast(e) => Ok(Result3::FailFast(e)),
            Self::RetryLater(m) => Err(m),
        }
    }
}

/// Round `x` to multiples of `n`.
///
/// ```
/// # use mini_haskell::utils::round_to;
/// assert_eq!(round_to(20, 42), 42);
/// assert_eq!(round_to(1120, 1024), 2048);
/// assert_eq!(round_to(2048, 32), 2048);
/// ```
#[inline]
pub const fn round_to(x: usize, n: usize) -> usize {
    (x + n - 1) / n * n
}

/// Lorem ipsum.
#[cfg(test)]
pub const LIPSUM: &str =
    "Lorem ipsum dolor sit amet, consectetur adipiscing elit. Vestibulum facilisis turpis ex, eu \
    dignissim purus varius non. Integer elit enim, rhoncus a lacinia sed, fermentum eget mauris. \
    Suspendisse bibendum pellentesque justo, et fermentum tortor tempor et. Sed interdum, ligula \
    quis sagittis tristique, mi magna malesuada felis, vitae gravida ligula libero ac tellus. \
    Morbi imperdiet scelerisque leo sit amet consequat. Pellentesque tellus lectus, sagittis in \
    gravida at, laoreet ut justo. Vivamus posuere arcu diam, eu pellentesque ante maximus \
    pulvinar. Phasellus id rhoncus enim, ut iaculis justo. Fusce interdum dolor vel purus pulvinar \
    aliquam. Curabitur nec nulla magna. Etiam sagittis sem nibh, eget auctor nunc molestie in. \
    Vivamus pretium augue in blandit porta. Integer tempus fermentum enim, non ultrices nulla \
    tempor quis. Sed vel tincidunt enim, at vulputate risus. Nulla facilisi. Ut pellentesque \
    pharetra urna ac finibus. Aenean ac dignissim orci. Praesent vulputate massa a vulputate \
    facilisis. Phasellus sed.";

macro_rules! method {
    ($f: ident) => {
        |x| x.$f()
    };
}

macro_rules! lexemes {
    { $($ps: tt)* } => {
        lexeme_types! { $($ps)* }
        lexeme_concrete! { $($ps)* }
    }
}

macro_rules! lexeme_types {
    { $( $(#[$meta: meta])* $l: ident $(($($t: ty),*))? ),* $(,)? } => {
        /// Lexeme type labels.
        #[derive(Copy, Clone, Eq, PartialEq, Debug)]
        pub enum LexemeType {
            $( $(#[$meta])* $l ),*
        }
    }
}

macro_rules! wildcard_from {
    ($($t: tt)*) => { .. }
}

macro_rules! lexeme_concrete {
    { $( $(#[$meta: meta])* $l: ident $(($($t: ty),*))? ),* $(,)? } => {
        /// Concrete lexeme type.
        #[derive(Clone, Eq, PartialEq, Debug)]
        pub enum Lexeme {
            $( $(#[$meta])* $l $(($($t),*))? ),*
        }
        impl Lexeme {
            /// Get lexeme type from a concrete lexeme.
            pub fn get_type(&self) -> LexemeType {
                match self {
                    $( Lexeme::$l $((wildcard_from!($($t),*)))? => LexemeType::$l ),*
                }
            }
        }
    }
}

#[cfg(all(test, feature = "log"))]
mod log_init {
    use std::sync::Once;

    static LOG_INIT: Once = Once::new();

    pub fn setup_logger() {
        LOG_INIT.call_once(|| env_logger::Builder::new()
            .format_level(true)
            .format_indent(Some(4))
            .format_timestamp(None)
            .format_module_path(true)
            .filter_level(log::LevelFilter::Trace)
            .target(env_logger::Target::Stdout)
            .write_style(env_logger::WriteStyle::Always)
            .init())
    }
}

#[cfg(all(test, feature = "log"))]
pub use log_init::setup_logger;

#[cfg(all(test, not(feature = "log")))]
pub fn setup_logger() {}

macro_rules! trace {
    (scanner, $($params: tt)+) => {
        #[cfg(feature = "scanner_trace")]
        log::trace!(target: "scanner", $($params)+);
    }
}
