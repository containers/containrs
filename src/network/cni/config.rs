//! CNI network config types
use crate::network::cni::exec::{ArgsBuilder, Exec};
use anyhow::{bail, Context, Result};
use derive_builder::Builder;
use getset::Getters;
use log::trace;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    convert::Into,
    fs::File,
    path::{Path, PathBuf},
};

#[derive(Clone, Debug, Builder, Getters)]
#[builder(pattern = "owned", setter(into))]
/// The CNI plugin definition.
pub struct Config<T>
where
    T: Default,
{
    #[getset(get = "pub")]
    /// The name of the plugin.
    name: String,

    #[getset(get = "pub")]
    /// The path to the configuration file.
    file: PathBuf,

    #[getset(get = "pub")]
    /// The configuration list.
    list: ConfigListFile,

    #[get]
    #[builder(default = "T::default()")]
    /// CNI command execution helper.
    exec: T,
}

impl<T> Config<T>
where
    T: Clone + Default + Exec,
{
    /// Verifies that a given plugin `name` is supported by the provided `version`.
    pub async fn validate(&self) -> Result<()> {
        let version = self
            .list()
            .cni_version()
            .as_ref()
            .context("no config `cniVersion` provided")?;

        let args = ArgsBuilder::default()
            .command("VERSION")
            .build()
            .context("build CNI exec args")?;

        for plugin in self.list().plugins() {
            let binary = which::which(plugin.typ())
                .with_context(|| format!("find plugin binary {} in $PATH", plugin.typ()))?;
            trace!("Using plugin binary {}", binary.display());

            trace!("Using CNI args {:?}", args);

            let output = self
                .exec()
                .run(&binary, &args)
                .await
                .context("exec CNI plugin")?;
            trace!("Got CNI ouput {}", output);

            if !serde_json::from_str::<VersionResult>(&output)
                .context("unmarshal CNI output")?
                .supported_versions()
                .contains(&version.into())
            {
                bail!(
                    "plugin {} does not support config version {}",
                    plugin.typ(),
                    version
                )
            }
        }

        Ok(())
    }
}

#[derive(Clone, Serialize, Deserialize, Debug, Default, Getters)]
/// VersionResult is the result which is returned when calling the `VERSION` CNI command.
pub struct VersionResult {
    #[get]
    #[serde(rename = "cniVersion")]
    /// The current CNI version of this plugin.
    current: String,

    #[get]
    #[serde(rename = "supportedVersions")]
    /// All supported versions by this plugin.
    supported_versions: Vec<String>,
}

#[derive(Clone, Serialize, Deserialize, Debug, Default, Builder, Getters)]
#[builder(default, pattern = "owned", setter(into, strip_option))]
// Config describes a CNI network configuration.
pub struct ConfigFile {
    #[getset(get = "pub")]
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "cniVersion"
    )]
    cni_version: Option<String>,

    #[getset(get = "pub")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    name: Option<String>,

    #[getset(get = "pub")]
    #[serde(rename = "type")]
    typ: String,

    #[getset(get = "pub")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    capabilities: Option<HashMap<String, bool>>,

    #[getset(get = "pub")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    ipam: Option<IPAM>,

    #[getset(get = "pub")]
    #[serde(default)]
    dns: DNS,

    #[getset(get = "pub")]
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "prevResult"
    )]
    raw_prev_result: Option<HashMap<String, Vec<u8>>>,
}

impl ConfigFile {
    /// Load a new ConfigFile from the provided file `Path`.
    pub fn from(path: &Path) -> Result<Self> {
        let file = File::open(path).with_context(|| format!("open file {}", path.display()))?;
        let mut config: Self = serde_json::from_reader(file)
            .with_context(|| format!("deserialize CNI config from file {}", path.display()))?;

        if config.name().is_none() {
            config.name = Some(config.typ().clone());
        }

        Ok(config)
    }
}

impl Into<ConfigListFile> for ConfigFile {
    fn into(self) -> ConfigListFile {
        ConfigListFile {
            cni_version: self.cni_version.clone(),
            name: self.name.clone(),
            plugins: vec![self],
            ..Default::default()
        }
    }
}

#[derive(Clone, Serialize, Deserialize, Debug, Default, Builder, Getters)]
#[builder(default, pattern = "owned", setter(into, strip_option))]
// ConfigListFile describes an ordered list of network configurations.
pub struct ConfigListFile {
    #[getset(get = "pub")]
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "cniVersion"
    )]
    cni_version: Option<String>,

    #[getset(get = "pub")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    name: Option<String>,

    #[getset(get = "pub")]
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "disableCheck"
    )]
    disable_check: Option<bool>,

    #[getset(get = "pub")]
    plugins: Vec<ConfigFile>,
}

impl ConfigListFile {
    /// Load a new ConfigListFile from the provided file `Path`
    pub fn from(path: &Path) -> Result<Self> {
        let file = File::open(path).with_context(|| format!("open file {}", path.display()))?;
        serde_json::from_reader(file)
            .with_context(|| format!("deserialize CNI config list from file {}", path.display()))
    }
}

#[derive(Clone, Serialize, Deserialize, Debug, Default, Builder, Getters)]
#[builder(default, pattern = "owned", setter(into, strip_option))]
pub struct IPAM {
    #[getset(get = "pub")]
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "type")]
    typ: Option<String>,
}

#[derive(Clone, Serialize, Deserialize, Debug, Default, Builder, Getters)]
#[builder(default, pattern = "owned", setter(into, strip_option))]
/// DNS contains values interesting for DNS resolvers
pub struct DNS {
    #[getset(get = "pub")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    nameservers: Option<Vec<String>>,

    #[getset(get = "pub")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    domain: Option<String>,

    #[getset(get = "pub")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    search: Option<Vec<String>>,

    #[getset(get = "pub")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    options: Option<Vec<String>>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::network::cni::exec::Args;
    use async_trait::async_trait;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[derive(Clone, Default)]
    struct Mock {
        result: String,
    }

    const VERSION: &str = "0.4.0";

    #[async_trait]
    impl Exec for Mock {
        async fn run(&self, _binary: &Path, _args: &Args) -> Result<String> {
            Ok(self.result.clone())
        }
    }

    fn new_list() -> Result<ConfigListFile> {
        Ok(ConfigListFileBuilder::default()
            .cni_version(VERSION)
            .plugins(vec![ConfigFileBuilder::default().typ("ls").build()?])
            .build()?)
    }

    #[tokio::test]
    async fn config_validate_success() -> Result<()> {
        let mock = Mock {
            result: serde_json::to_string(&VersionResult {
                current: VERSION.into(),
                supported_versions: vec![VERSION.into()],
            })?,
        };
        let config = ConfigBuilder::<Mock>::default()
            .exec(mock)
            .name("name")
            .file("file")
            .list(new_list()?)
            .build()?;
        config.validate().await
    }

    #[tokio::test]
    async fn config_validate_failure_unsupported_version() -> Result<()> {
        let mock = Mock {
            result: serde_json::to_string(&VersionResult::default())?,
        };
        let config = ConfigBuilder::<Mock>::default()
            .exec(mock)
            .name("name")
            .file("file")
            .list(new_list()?)
            .build()?;
        assert!(config.validate().await.is_err());
        Ok(())
    }

    #[tokio::test]
    async fn config_validate_failure_wrong_json_result() -> Result<()> {
        let mock = Mock {
            result: "wrong".into(),
        };
        let config = ConfigBuilder::<Mock>::default()
            .exec(mock)
            .name("name")
            .file("file")
            .list(new_list()?)
            .build()?;
        assert!(config.validate().await.is_err());
        Ok(())
    }

    #[tokio::test]
    async fn config_validate_failure_wrong_type() -> Result<()> {
        let mock = Mock {
            result: "wrong".into(),
        };
        let config = ConfigBuilder::<Mock>::default()
            .exec(mock)
            .name("name")
            .file("file")
            .list(
                ConfigListFileBuilder::default()
                    .cni_version(VERSION)
                    .plugins(vec![ConfigFileBuilder::default()
                        .typ("/some/wrong/binary")
                        .build()?])
                    .build()?,
            )
            .build()?;
        assert!(config.validate().await.is_err());
        Ok(())
    }

    #[tokio::test]
    async fn config_validate_failure_no_version() -> Result<()> {
        let mock = Mock {
            result: "wrong".into(),
        };
        let config = ConfigBuilder::<Mock>::default()
            .exec(mock)
            .name("name")
            .file("file")
            .list(ConfigListFileBuilder::default().build()?)
            .build()?;
        assert!(config.validate().await.is_err());
        Ok(())
    }

    #[test]
    fn config_file_to_cnfig_list_file() -> Result<()> {
        let config = ConfigFileBuilder::default()
            .cni_version(VERSION)
            .name("name")
            .build()?;
        let config_list: ConfigListFile = config.into();
        assert_eq!(config_list.cni_version(), &Some(VERSION.into()));
        assert_eq!(config_list.name(), &Some("name".into()));
        Ok(())
    }

    #[test]
    fn config_file_from_path_success() -> Result<()> {
        let temp_file = NamedTempFile::new()?;
        temp_file.as_file().write_all(
            br#"{
                "cniVersion": "0.4.0",
                "name": "cni-network",
                "type": "bridge",
                "bridge": "cni0",
                "isGateway": true,
                "ipMasq": true,
                "hairpinMode": true,
                "ipam": {
                    "type": "host-local",
                    "routes": [
                        { "dst": "0.0.0.0/0" },
                        { "dst": "1100:200::1/24" }
                    ],
                    "ranges": [
                        [{ "subnet": "10.85.0.0/16" }],
                        [{ "subnet": "1100:200::/24" }]
                    ]
                }
              }"#,
        )?;
        let config_file = ConfigFile::from(temp_file.path())?;
        assert_eq!(config_file.cni_version(), &Some("0.4.0".into()));
        assert_eq!(config_file.name(), &Some("cni-network".into()));
        assert_eq!(
            config_file.ipam().as_ref().context("no ipam")?.typ(),
            &Some("host-local".into())
        );
        Ok(())
    }

    #[test]
    fn config_file_from_path_success_convert() -> Result<()> {
        let temp_file = NamedTempFile::new()?;
        temp_file.as_file().write_all(b"{\"type\": \"bridge\"}")?;
        let config_file = ConfigFile::from(temp_file.path())?;
        assert_eq!(config_file.name(), &Some("bridge".into()));
        Ok(())
    }

    #[test]
    fn config_file_from_path_failure_not_exists() {
        assert!(ConfigFile::from(&Path::new("")).is_err())
    }

    #[test]
    fn config_list_file_from_path_success() -> Result<()> {
        let temp_file = NamedTempFile::new()?;
        temp_file.as_file().write_all(
            br#"{
              "cniVersion": "0.4.0",
              "name": "name",
              "plugins": [
                {
                  "type": "bridge",
                  "bridge": "cni-name0",
                  "isGateway": true,
                  "ipMasq": true,
                  "hairpinMode": true,
                  "ipam": {
                    "type": "host-local",
                    "routes": [{ "dst": "0.0.0.0/0" }],
                    "ranges": [[{ "subnet": "10.88.0.0/16", "gateway": "10.88.0.1" }]]
                  }
                },
                { "type": "portmap", "capabilities": { "portMappings": true } },
                { "type": "firewall" },
                { "type": "tuning" }
              ]
            }"#,
        )?;
        let config_list_file = ConfigListFile::from(temp_file.path())?;
        assert_eq!(config_list_file.cni_version(), &Some("0.4.0".into()));
        assert_eq!(config_list_file.name(), &Some("name".into()));
        assert_eq!(config_list_file.plugins().len(), 4);
        Ok(())
    }

    #[test]
    fn config_list_file_from_path_failure_not_exists() {
        assert!(ConfigListFile::from(&Path::new("")).is_err())
    }
}
