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

/// Round `x` to multiples of `n`.
///
/// ```
/// # use mini_haskell::utils::round_to;
/// assert_eq!(round_to(20, 42), 42);
/// assert_eq!(round_to(1120, 1024), 2048);
/// assert_eq!(round_to(2048, 32), 2048);
/// ```
pub const fn round_to(x: usize, n: usize) -> usize {
    (x + n - 1) / n * n
}

/// Return the minimum and the maxinum.
///
/// ```
/// # use mini_haskell::utils::min_max;
/// assert_eq!((true, 1, 2), min_max(1, 2));
/// assert_eq!((false, 1, 2), min_max(2, 1));
/// ```
#[inline]
pub fn min_max<T: Ord>(x: T, y: T) -> (bool, T, T) {
    if x <= y {
        (true, x, y)
    } else {
        (false, y, x)
    }
}

/// Good old self-increment.
///
/// ```
/// # use mini_haskell::utils::inc;
/// let mut x = 42;
/// assert_eq!(inc(&mut x), 42);
/// assert_eq!(x, 43);
/// ```
pub fn inc(x: &mut usize) -> usize {
    let res = *x;
    *x += 1;
    res
}

/// Duplicate a mutable reference.
///
/// ```
/// # use mini_haskell::utils::dup_mut;
/// let mut x = 42;
/// let r = &mut x;
/// let r2 = unsafe { dup_mut(&r) };
/// assert_eq!(r as *mut _, r2 as *mut _);
/// ```
#[inline]
pub unsafe fn dup_mut<'a, T>(r: &&'a mut T) -> &'a mut T {
    &mut *(*r as *const T as *mut T)
}

/// If greater, return the difference, else `None` is returned.
pub fn greater(x: usize, y: usize) -> Option<usize> {
    if x > y { Some(x - y) } else { None }
}

