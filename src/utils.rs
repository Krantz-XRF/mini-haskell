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
#[inline]
pub const fn round_to(x: usize, n: usize) -> usize {
    (x + n - 1) / n * n
}

/// Lorem ipsum.
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
