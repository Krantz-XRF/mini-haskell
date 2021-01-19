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

//! Control flow utilities.

/// The uninhabited type `Void`.
#[derive(Copy, Clone, Debug)]
pub enum Void {}

impl Void {
    /// Consumes a [`Void`] value, serves as an unreachable.
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
/// Use this for success/failure semantics, since [`Either`] is used to model the control flow.
pub trait Maybe {
    /// The success type in a `Just`.
    type Just;
    /// Construct a success.
    fn just(x: Self::Just) -> Self;
    /// Check whether it is a success.
    fn is_just(&self) -> bool;
    /// Check whether it is a failure.
    fn is_nothing(&self) -> bool { !self.is_just() }
    /// Convert into an [`Option`].
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

/// The [`std::ops::Try`] trait is not yet stable. We roll up our own for now.
/// It is named after `Either`, `Left`, and `Right` from Haskell.
pub trait Either {
    /// The type to propagate in a `Left`.
    type Left;
    /// The type to continue with in a `Right`.
    type Right;
    /// Construct a `Left`:
    /// - [`None`] for [`Option`]
    /// - [`Err`] for [`Result`]
    fn left(x: Self::Left) -> Self;
    /// Construct a `Right`:
    /// - [`Some`] for [`Option`]
    /// - [`Ok`] for [`Result`]
    fn right(x: Self::Right) -> Self;
    /// Consumes the value and makes a [`Result`].
    ///
    /// Note that by design (to follow the convention in Haskell), the
    /// [`Err`] is for `Left` and the [`Ok`] is for `Right`.
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
