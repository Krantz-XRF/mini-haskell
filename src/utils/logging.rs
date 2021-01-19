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

//! Logging utilities.

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
