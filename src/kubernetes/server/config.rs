//! Configuration related structures
use clap::{crate_name, crate_version, AppSettings, Clap};
use derive_builder::Builder;
use getset::{CopyGetters, Getters};
use lazy_static::lazy_static;
use log::LevelFilter;
use nix::unistd::{self, Uid};
use serde::{Deserialize, Serialize};
use std::{env, path::PathBuf};
use strum::{AsRefStr, EnumString};

lazy_static! {
    static ref DEFAULT_SOCK_PATH: String = Config::default_sock_path().display().to_string();
    static ref DEFAULT_STORAGE_PATH: String = Config::default_storage_path().display().to_string();
    static ref DEFAULT_CNI_PLUGIN_PATHS: String =
        env::var("PATH").unwrap_or_else(|_| "/opt/cni/bin".into());
}

#[derive(Builder, Clap, CopyGetters, Getters, Deserialize, Serialize)]
#[builder(default, pattern = "owned", setter(into, strip_option))]
#[serde(rename_all = "kebab-case")]
#[clap(
    about("CRI - Container Runtime library for Kubernetes (CRI) and friends"),
    after_help("More info at: https://github.com/cri-o/cri"),
    global_setting(AppSettings::ColoredHelp),
    version(crate_version!()),
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
    /// The logging level of the application.
    log_level: LevelFilter,

    #[get_copy = "pub"]
    #[clap(
        default_value("lib"),
        env("CRI_LOG_SCOPE"),
        long("log-scope"),
        possible_values(&[LogScope::Lib.as_ref(), LogScope::Global.as_ref()]),
        value_name("SCOPE")
    )]
    /// The logging scope of the application. If set to `global`, then all dependent crates will
    /// log on the provided level, too. Otherwise the logs are scoped to this application only.
    log_scope: LogScope,

    #[get = "pub"]
    #[clap(
        default_value(&DEFAULT_SOCK_PATH),
        env("CRI_SOCK_PATH"),
        long("sock-path"),
        value_name("PATH")
    )]
    /// The path to the unix socket for the server.
    sock_path: PathBuf,

    #[get = "pub"]
    #[clap(
        default_value(&DEFAULT_STORAGE_PATH),
        env("CRI_STORAGE_PATH"),
        long("storage-path"),
        value_name("PATH")
    )]
    /// The path to the persistent storage for the server.
    storage_path: PathBuf,

    #[get = "pub"]
    #[clap(
        env("CRI_CNI_DEFAULT_NETWORK"),
        long("cni-default-network"),
        value_name("NAME")
    )]
    /// The default CNI network name to choose.
    cni_default_network: Option<String>,

    #[get = "pub"]
    #[clap(
        default_value("/etc/cni/net.d"),
        env("CRI_CNI_CONFIG_PATHS"),
        long("cni-config-paths"),
        value_name("PATH")
    )]
    /// The paths to the CNI configurations.
    cni_config_paths: Vec<PathBuf>,

    #[get = "pub"]
    #[clap(
        default_value(&DEFAULT_CNI_PLUGIN_PATHS),
        env("CRI_CNI_PLUGIN_PATHS"),
        long("cni-plugin-paths"),
        value_name("PATH")
    )]
    /// The paths to the CNI plugin binaries, separated by the OS typic separator.
    cni_plugin_paths: String,
}

impl Config {
    /// Return the default socket path depending if running as root or not.
    fn default_sock_path() -> PathBuf {
        Self::default_run_path(unistd::getuid())
            .join(crate_name!())
            .with_extension("sock")
    }

    /// Return the default storage path depending if running as root or not.
    fn default_storage_path() -> PathBuf {
        Self::default_run_path(unistd::getuid()).join("storage")
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

#[derive(AsRefStr, Clone, Copy, Debug, Deserialize, EnumString, PartialEq, Serialize)]
#[strum(serialize_all = "snake_case")]
/// Defines the scope of the log level
pub enum LogScope {
    /// Logging will only happen on a library level.
    Lib,

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
        assert!(c.cni_default_network().is_none());
        assert_eq!(c.cni_config_paths().len(), 1);
        assert!(!c.cni_plugin_paths().is_empty());
    }

    #[test]
    fn build_config() -> Result<()> {
        let c = ConfigBuilder::default()
            .log_level(LevelFilter::Warn)
            .sock_path("/some/path")
            .cni_default_network("default-network")
            .cni_config_paths(["a", "b"].iter().map(PathBuf::from).collect::<Vec<_>>())
            .cni_plugin_paths("1:2:3")
            .log_scope(LogScope::Global)
            .storage_path("/some/other/path")
            .build()?;

        assert_eq!(c.log_level(), LevelFilter::Warn);
        assert_eq!(&c.sock_path().display().to_string(), "/some/path");
        assert_eq!(c.log_scope(), LogScope::Global);
        assert_eq!(&c.storage_path().display().to_string(), "/some/other/path");
        assert_eq!(c.cni_default_network(), &Some("default-network".into()));
        assert_eq!(c.cni_config_paths().len(), 2);
        assert_eq!(c.cni_plugin_paths(), "1:2:3");

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

    #[test]
    fn default_storage_path() {
        assert!(Config::default_storage_path()
            .display()
            .to_string()
            .contains("storage"));
    }
}
