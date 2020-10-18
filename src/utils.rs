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

/// If greater, return the difference, else `None` is returned.
pub fn greater(x: usize, y: usize) -> Option<usize> {
    if x > y { Some(x - y) } else { None }
}

/// Lorem ipsum.
pub const LIPSUM: &'static str =
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

#[cfg(test)]
macro_rules! assert_eq_iter {
    ($x: expr, $y: expr) => {
        assert!($x.eq($y));
    }
}

#[cfg(test)]
macro_rules! assert_eq_str {
    ($x: expr, $y: expr) => {
        assert_eq_iter!($x, $y.chars());
    }
}
