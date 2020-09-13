//! Configuration related structures
use clap::{AppSettings, Clap};
use getset::{CopyGetters, Getters};
use log::LevelFilter;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Clap, CopyGetters, Getters, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
#[clap(
    after_help("More info at: https://github.com/cri-o/cri"),
    global_setting(AppSettings::ColoredHelp)
)]
/// CRI - The Kubernetes Container Runtime written in Rust
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

    #[get = "pub"]
    #[clap(
        default_value("/var/run/cri/cri.sock"),
        env("CRI_SOCK_PATH"),
        long("sock-path")
    )]
    /// The path to the unix socket for the server
    sock_path: PathBuf,
}

impl Default for Config {
    fn default() -> Self {
        Self::parse()
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    fn default_config() {
        let c = Config::default();
        assert_eq!(c.log_level(), LevelFilter::Info);
    }
}
