//! Configuration related structures
use clap::{crate_name, AppSettings, Clap};
use derive_builder::Builder;
use getset::{CopyGetters, Getters};
use lazy_static::lazy_static;
use log::LevelFilter;
use nix::unistd::{self, Uid};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use strum::EnumString;

lazy_static! {
    static ref DEFAULT_SOCK_PATH: String = Config::default_sock_path().display().to_string();
}

#[derive(Builder, Clap, CopyGetters, Getters, Deserialize, Serialize)]
#[builder(default, pattern = "owned", setter(into))]
#[serde(rename_all = "kebab-case")]
#[clap(
    about("CRI - The Kubernetes Container Runtime written in Rust"),
    after_help("More info at: https://github.com/cri-o/cri"),
    global_setting(AppSettings::ColoredHelp)
)]
/// Config is the main configuration structure for the server.
pub struct Config {
    #[get_copy = "pub"]
    #[clap(
        default_value("info"),
        env("CRI_LOG_LEVEL"),
        long("log-level"),
        possible_values(&["trace", "debug", "info", "warn", "error", "off"]),
        short('l'),
        value_name("LEVEL")
    )]
    /// The logging level of the application
    log_level: LevelFilter,

    #[get_copy = "pub"]
    #[clap(
        default_value("lib"),
        env("CRI_LOG_SCOPE"),
        long("log-scope"),
        possible_values(&["lib", "global"]),
        value_name("SCOPE")
    )]
    /// The logging scope of the application. If set to `global`, then all dependent crates will
    /// log on the provided level, too. Otherwise the logs are scoped to this application only.
    log_scope: LogScope,

    #[get = "pub"]
    #[clap(
        default_value(&DEFAULT_SOCK_PATH),
        env("CRI_SOCK_PATH"),
        long("sock-path")
    )]
    /// The path to the unix socket for the server
    sock_path: PathBuf,
}

impl Config {
    /// Return the default socket path depending if running as root or not.
    fn default_sock_path() -> PathBuf {
        Self::default_run_path(unistd::getuid())
            .join(crate_name!())
            .with_extension("sock")
    }

    /// Return the default run path depending on the provided user ID.
    fn default_run_path(uid: Uid) -> PathBuf {
        if uid.is_root() {
            PathBuf::from("/var/run/").join(crate_name!())
        } else {
            PathBuf::from("/var/run/user")
                .join(uid.to_string())
                .join(crate_name!())
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::parse()
    }
}

#[derive(Clone, Copy, Debug, Deserialize, EnumString, PartialEq, Serialize)]
/// Defines the scope of the log level
pub enum LogScope {
    #[strum(serialize = "lib")]
    /// Logging will only happen on a library level.
    Lib,

    #[strum(serialize = "global")]
    /// All dependent libraries will log too.
    Global,
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use anyhow::Result;

    #[test]
    fn default_config() {
        let c = Config::default();
        assert_eq!(c.log_level(), LevelFilter::Info);
    }

    #[test]
    fn build_config() -> Result<()> {
        let c = ConfigBuilder::default()
            .log_level(LevelFilter::Warn)
            .sock_path("/some/path")
            .log_scope(LogScope::Global)
            .build()?;

        assert_eq!(c.log_level(), LevelFilter::Warn);
        assert_eq!(&c.sock_path().display().to_string(), "/some/path");
        assert_eq!(c.log_scope(), LogScope::Global);

        Ok(())
    }

    #[test]
    fn default_run_path_root() {
        let uid = Uid::from_raw(0);
        assert!(uid.is_root());
        assert!(!Config::default_run_path(uid)
            .display()
            .to_string()
            .contains("user"));
    }

    #[test]
    fn default_run_path_non_root() {
        let uid = Uid::from_raw(1000);
        assert!(!uid.is_root());
        assert!(Config::default_run_path(uid)
            .display()
            .to_string()
            .contains(&uid.to_string()));
    }

    #[test]
    fn default_sock_path() {
        assert!(Config::default_sock_path()
            .display()
            .to_string()
            .contains(".sock"));
    }
}
