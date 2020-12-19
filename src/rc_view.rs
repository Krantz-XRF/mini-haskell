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

//! View into an [`Rc`], focus on a part of the whole data.

use std::ops::Deref;
use std::ptr::NonNull;
use std::rc::Rc;
use std::cell::UnsafeCell;
use std::fmt::Formatter;

/// A view into an [`Rc`].
pub struct RcView<T: ?Sized, U: ?Sized> {
    #[allow(dead_code)]
    whole: Rc<T>,
    focus: NonNull<U>,
}

impl<T: ?Sized> From<Rc<T>> for RcView<T, T> {
    fn from(whole: Rc<T>) -> Self {
        RcView {
            focus: NonNull::from(whole.as_ref()),
            whole,
        }
    }
}

impl<T: ?Sized, U: std::fmt::Debug + ?Sized> std::fmt::Debug for RcView<T, U> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result { self.deref().fmt(f) }
}

impl<T: ?Sized, U: std::fmt::Display + ?Sized> std::fmt::Display for RcView<T, U> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result { self.deref().fmt(f) }
}

impl<T: ?Sized, U: ?Sized> RcView<T, U> {
    /// Create a view into an [`Rc`].
    pub fn new(whole: Rc<T>, to_focus: impl FnOnce(&T) -> &U) -> Self {
        let focus = NonNull::from(to_focus(&whole));
        RcView { whole, focus }
    }

    /// Create a view into an [`Rc`].
    ///
    /// # Safety
    ///
    /// Call on this unsafe functions should take special care so that the provided
    /// reference `focus` is indeed a view into the related [`Rc`].
    pub unsafe fn wrap(whole: Rc<T>, focus: &U) -> Self {
        RcView {
            whole,
            focus: NonNull::from(focus),
        }
    }

    /// Refocus the view.
    pub fn map<V: ?Sized>(self, f: impl FnOnce(&U) -> &V) -> RcView<T, V> {
        RcView {
            focus: NonNull::from(f(unsafe { self.focus.as_ref() })),
            whole: self.whole,
        }
    }

    /// Derive an [`RcView`] to a new focus from this view.
    ///
    /// # Safety
    ///
    /// Call on this unsafe functions should take special care so that the provided
    /// reference `focus` is indeed a view into the related [`Rc`].
    pub unsafe fn derive<V: ?Sized>(&self, focus: &V) -> RcView<T, V> {
        RcView::<T, V>::wrap(self.whole.clone(), focus)
    }

    /// Derive an [`RcView`] to a new focus from this view. Consumes this view.
    ///
    /// # Safety
    ///
    /// Call on this unsafe functions should take special care so that the provided
    /// reference `focus` is indeed a view into the related [`Rc`].
    pub unsafe fn derive_take<V: ?Sized>(self, focus: &V) -> RcView<T, V> {
        RcView::<T, V>::wrap(self.whole, focus)
    }
}

impl<T: ?Sized, U: ?Sized> Deref for RcView<T, U> {
    type Target = U;
    fn deref(&self) -> &Self::Target {
        unsafe { self.focus.as_ref() }
    }
}

impl<T: ?Sized, U> From<RcView<UnsafeCell<T>, Option<U>>> for Option<U> {
    fn from(mut this: RcView<UnsafeCell<T>, Option<U>>) -> Self {
        unsafe { this.focus.as_mut() }.take()
    }
}
