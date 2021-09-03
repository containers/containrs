//! Logging faccade for the FFI

use crate::ffi::error::update_last_err_if_required;
use anyhow::{Context, Result};
use clap::crate_name;
use std::env;
use strum::AsRefStr;

#[repr(C)]
#[allow(dead_code)]
#[derive(Debug, AsRefStr)]
#[strum(serialize_all = "snake_case")]
/// An enum representing the available verbosity level filters of the logger.
pub enum LogLevel {
    /// A level lower than all log levels.
    Off,

    /// Corresponds to the `Error` log level.
    Error,

    /// Corresponds to the `Warn` log level.
    Warn,

    /// Corresponds to the `Info` log level.
    Info,

    /// Corresponds to the `Debug` log level.
    Debug,

    /// Corresponds to the `Trace` log level.
    Trace,
}

#[no_mangle]
/// Init the log level by the provided level string.
/// Populates the last error on any failure.
pub extern "C" fn log_init(level: LogLevel) {
    update_last_err_if_required(log_init_res(level))
}

fn log_init_res(level: LogLevel) -> Result<()> {
    env::set_var("RUST_LOG", format!("{}={}", crate_name!(), level.as_ref()));
    env_logger::try_init().context("init log level")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ffi::error::last_error_length;

    #[test]
    fn log_init_success() {
        log_init(LogLevel::Error);
        assert_eq!(last_error_length(), 0);
    }

    #[test]
    fn log_level() {
        assert_eq!(LogLevel::Error.as_ref(), "error");
        assert_eq!(LogLevel::Debug.as_ref(), "debug");
    }
}
