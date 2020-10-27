//! CNI network config types
use crate::network::cni::{
    exec::{DefaultExec, Exec},
    plugin::PluginBuilder,
};
use anyhow::{bail, Context, Result};
use derive_builder::Builder;
use getset::{Getters, Setters};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    convert::Into,
    fmt,
    fs::File,
    io::Read,
    path::{Path, PathBuf},
};

#[derive(Clone, Builder, Getters, Setters)]
#[builder(pattern = "owned", setter(into))]
/// The CNI plugin definition.
pub struct Config {
    #[getset(get = "pub")]
    /// The name of the plugin.
    name: String,

    #[getset(get = "pub")]
    /// The path to the configuration file.
    file: PathBuf,

    #[getset(get = "pub")]
    /// The configuration list.
    list: ConfigListFile,

    #[getset(get, set = "pub")]
    #[builder(default = "Box::new(DefaultExec)")]
    /// CNI command execution helper.
    plugin_exec: Box<dyn Exec>,
}

impl fmt::Debug for Config {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Config")
            .field("name", self.name())
            .field("file", self.file())
            .field("list", self.list())
            .finish()
    }
}

impl Config {
    /// Verifies that a given plugin `name` is supported by the provided `version`.
    pub async fn validate(&self, plugin_paths: &str) -> Result<()> {
        let version = self
            .list()
            .cni_version()
            .as_ref()
            .context("no config `cniVersion` provided")?;

        for plugin_config in self.list().plugins() {
            let mut plugin = PluginBuilder::default()
                .binary(plugin_config.typ())
                .build()
                .context("build CNI plugin")?;
            plugin.set_exec(self.plugin_exec.clone());

            if !plugin
                .find_binary(plugin_paths)
                .context("find plugin binary")?
                .version()
                .await
                .context("get plugin version")?
                .supported_versions()
                .contains(&version.into())
            {
                bail!(
                    "plugin {} does not support config version {}",
                    plugin_config.typ(),
                    version
                )
            }
        }

        Ok(())
    }
}

#[derive(Clone, Serialize, Deserialize, Default, Builder, Getters)]
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
    #[serde(default)]
    raw: Vec<u8>,
}

impl fmt::Debug for ConfigFile {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Config")
            .field("cni_version", self.cni_version())
            .field("name", self.name())
            .field("type", self.typ())
            .field("capabilities", self.capabilities())
            .field("ipam", self.ipam())
            .field("dns", self.dns())
            .finish()
    }
}

impl ConfigFile {
    /// Load a new ConfigFile from the provided file `Path`.
    pub fn from(path: &Path) -> Result<Self> {
        // Read the file.
        let mut raw = Vec::new();
        File::open(path)
            .with_context(|| format!("open file {}", path.display()))?
            .read_to_end(&mut raw)?;

        // Deserialize the config.
        let mut config: Self = serde_json::from_slice(&raw)
            .with_context(|| format!("deserialize CNI config from file {}", path.display()))?;

        // Save the raw content as well for later passing to the CNI plugin.
        config.raw = raw;

        // Use a fallback name if necessary.
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
    use crate::network::cni::plugin::{tests::ExecMock, VersionResult};
    use std::io::Write;
    use tempfile::NamedTempFile;

    const VERSION: &str = "0.4.0";

    fn new_list() -> Result<ConfigListFile> {
        Ok(ConfigListFileBuilder::default()
            .cni_version(VERSION)
            .plugins(vec![ConfigFileBuilder::default().typ("ls").build()?])
            .build()?)
    }

    #[tokio::test]
    async fn config_validate_success() -> Result<()> {
        let mock = ExecMock::boxed()?;
        let mut config = ConfigBuilder::default()
            .name("name")
            .file("file")
            .list(new_list()?)
            .build()?;
        config.set_plugin_exec(mock);
        config.validate("").await
    }

    #[tokio::test]
    async fn config_validate_failure_unsupported_version() -> Result<()> {
        let mut mock = ExecMock::boxed()?;
        mock.result = Ok(serde_json::to_string(&VersionResult::default())?);
        let mut config = ConfigBuilder::default()
            .name("name")
            .file("file")
            .list(new_list()?)
            .build()?;
        config.set_plugin_exec(mock);
        assert!(config.validate("").await.is_err());
        Ok(())
    }

    #[tokio::test]
    async fn config_validate_failure_wrong_json_result() -> Result<()> {
        let mut mock = ExecMock::boxed()?;
        mock.result = Ok("wrong".into());
        let mut config = ConfigBuilder::default()
            .name("name")
            .file("file")
            .list(new_list()?)
            .build()?;
        config.set_plugin_exec(mock);
        assert!(config.validate("").await.is_err());
        Ok(())
    }

    #[tokio::test]
    async fn config_validate_failure_wrong_type() -> Result<()> {
        let mock = ExecMock::boxed()?;
        let mut config = ConfigBuilder::default()
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
        config.set_plugin_exec(mock);
        assert!(config.validate("").await.is_err());
        Ok(())
    }

    #[tokio::test]
    async fn config_validate_failure_no_version() -> Result<()> {
        let mock = ExecMock::boxed()?;
        let mut config = ConfigBuilder::default()
            .name("name")
            .file("file")
            .list(ConfigListFileBuilder::default().build()?)
            .build()?;
        config.set_plugin_exec(mock);
        assert!(config.validate("").await.is_err());
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
