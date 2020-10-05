//! CNI plugin helpers mostly around their execution

use crate::network::cni::{
    config::DNS,
    exec::{ArgsBuilder, DefaultExec, Exec},
};
use anyhow::{format_err, Context, Result};
use derive_builder::Builder;
use getset::{Getters, Setters};
use ipnetwork::IpNetwork;
use log::trace;
use serde::{Deserialize, Serialize};
use std::{env, fmt::Debug, net::IpAddr, path::PathBuf};
use strum::{AsRefStr, EnumString, IntoStaticStr};

#[derive(Builder, Getters, Setters)]
#[builder(pattern = "owned", setter(into))]
pub struct Plugin {
    #[get]
    /// Path to the plugin binary.
    binary: PathBuf,

    #[getset(get, set = "pub")]
    #[builder(default = "Box::new(DefaultExec)")]
    /// CNI command execution helper.
    exec: Box<dyn Exec>,
}

#[derive(AsRefStr, IntoStaticStr, Copy, Clone, Debug, EnumString, Eq, PartialEq)]
#[strum(serialize_all = "shouty_snake_case")]
enum Command {
    Add,
    Del,
    Version,
}

impl Plugin {
    /// Find the plugin binary in $PATH.
    pub fn find_binary(mut self, plugin_paths: &str) -> Result<Self> {
        self.binary = which::which_in(
            self.binary(),
            Some(if plugin_paths.is_empty() {
                env::var("PATH")?
            } else {
                plugin_paths.into()
            }),
            env::current_dir().context("get current working directory")?,
        )
        .with_context(|| {
            format!(
                "find plugin binary {} in paths: {}",
                self.binary().display(),
                plugin_paths
            )
        })?;
        trace!("Using plugin binary {}", self.binary.display());
        Ok(self)
    }

    /// Create a version request for the plugin.
    pub async fn version(&self) -> Result<VersionResult> {
        let args = ArgsBuilder::default()
            .command(Command::Version.as_ref())
            .build()
            .context("build CNI exec args")?;
        trace!("Using CNI args {:?}", args);

        let output = self
            .exec()
            .run(self.binary(), &args)
            .await
            .context("exec CNI plugin")?;
        trace!("Got CNI ouput {}", output.trim());

        serde_json::from_str::<VersionResult>(&output).context("unmarshal CNI output")
    }

    /// Add a network via the plugin.
    pub async fn add(
        &self,
        container_id: &str,
        network_namespace_path: &str,
        interface_name: &str,
        raw_cni_config: &[u8],
    ) -> Result<CNIResult> {
        Ok(self
            .cmd(
                Command::Add,
                container_id,
                network_namespace_path,
                interface_name,
                raw_cni_config,
            )
            .await?
            .context("no CNI result")?)
    }

    /// Delete a network via the plugin.
    pub async fn del(
        &self,
        container_id: &str,
        network_namespace_path: &str,
        interface_name: &str,
        raw_cni_config: &[u8],
    ) -> Result<()> {
        self.cmd(
            Command::Del,
            container_id,
            network_namespace_path,
            interface_name,
            raw_cni_config,
        )
        .await?;
        Ok(())
    }

    /// Run a command with the provided network namespace path and container ID.
    async fn cmd(
        &self,
        command: Command,
        container_id: &str,
        network_namespace_path: &str,
        interface_name: &str,
        raw_cni_config: &[u8],
    ) -> Result<Option<CNIResult>> {
        let args = ArgsBuilder::default()
            .command(command.as_ref())
            .container_id(container_id)
            .network_namespace(network_namespace_path)
            .interface_name(interface_name)
            .path(
                self.binary()
                    .parent()
                    .context("binary has no parent path")?
                    .display()
                    .to_string(),
            )
            .build()
            .context("build CNI exec args")?;
        trace!("Using CNI args {:?}", args);

        match self
            .exec()
            .run_with_stdin(self.binary(), &args, raw_cni_config)
            .await
        {
            Ok(output) if command == Command::Add => {
                let result =
                    serde_json::from_str::<CNIResult>(&output).context("unmarshal CNI result")?;
                trace!("Got CNI ouput {:?}", result);
                Ok(Some(result))
            }
            Ok(_) => Ok(None),
            Err(e) => {
                let cni_error = serde_json::from_str::<ErrorResult>(&e.to_string())
                    .context("unmarshal CNI error")?;
                Err(format_err!("CNI error: {}", cni_error.message()))
            }
        }
    }
}

#[derive(Clone, Serialize, Deserialize, Debug, Default, Getters)]
/// VersionResult is the result which is returned when calling the `VERSION` CNI command.
pub struct VersionResult {
    #[get = "pub"]
    #[serde(rename = "cniVersion")]
    /// The current CNI version of this plugin.
    current: String,

    #[get = "pub"]
    #[serde(rename = "supportedVersions")]
    /// All supported versions by this plugin.
    supported_versions: Vec<String>,
}

#[derive(Clone, Serialize, Deserialize, Debug, Default, Getters)]
/// CNIResult gets returned from the plugin (via stdout) to the caller.
pub struct CNIResult {
    #[get = "pub"]
    #[serde(rename = "cniVersion")]
    /// The current CNI version of this plugin.
    cni_version: String,

    #[get = "pub"]
    #[serde(default)]
    /// The list of network interfaces.
    interfaces: Vec<NetworkInterface>,

    #[get = "pub"]
    #[serde(default)]
    /// The list of IPs.
    ips: Vec<IP>,

    #[get = "pub"]
    #[serde(default)]
    /// The list of routes.
    routes: Vec<Route>,

    #[get = "pub"]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// The DNS configuration.
    dns: Option<DNS>,
}

#[derive(Clone, Serialize, Deserialize, Debug, Default, Getters)]
/// Interface contains values about the created interfaces.
pub struct NetworkInterface {
    #[get = "pub"]
    name: String,

    #[get = "pub"]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    mac: Option<String>,

    #[get = "pub"]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    sandbox: Option<String>,
}

#[derive(Clone, Serialize, Deserialize, Debug, Getters)]
/// IP contains values necessary to configure an IP address on an interface.
pub struct IP {
    #[get = "pub"]
    /// IP version, either "4" or "6"
    version: String,

    #[get = "pub"]
    /// Index into Result structs Interfaces list
    interface: usize,

    #[get = "pub"]
    address: IpNetwork,

    #[get = "pub"]
    gateway: IpAddr,
}

#[derive(Clone, Serialize, Deserialize, Debug, Getters)]
pub struct Route {
    #[get = "pub"]
    dst: IpNetwork,

    #[get = "pub"]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    gw: Option<IpAddr>,
}

#[derive(Clone, Serialize, Deserialize, Debug, Default, Getters)]
/// Error gets returned in case the CNI plugin command fails.
pub struct ErrorResult {
    #[get = "pub"]
    /// Error code.
    code: u64,

    #[get = "pub"]
    #[serde(rename = "msg")]
    /// Error message.
    message: String,
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::network::cni::exec::Args;
    use anyhow::{bail, format_err};
    use async_trait::async_trait;
    use std::path::Path;

    pub struct ExecMock {
        pub result: Result<String>,
    }

    impl Clone for ExecMock {
        fn clone(&self) -> Self {
            let result = match &self.result {
                Ok(s) => Ok(s.clone()),
                Err(e) => Err(format_err!("{}", e)),
            };
            Self { result }
        }
    }

    #[async_trait]
    impl Exec for ExecMock {
        async fn run(&self, _binary: &Path, _args: &Args) -> Result<String> {
            self.result()
        }

        async fn run_with_stdin(
            &self,
            _binary: &Path,
            _args: &Args,
            _stdin: &[u8],
        ) -> Result<String> {
            self.result()
        }
    }

    const VERSION: &str = "0.4.0";

    impl ExecMock {
        /// Create a new boxed executor mock.
        pub fn boxed() -> Result<Box<Self>> {
            Ok(Box::new(Self {
                result: Ok(serde_json::to_string(&Self::default_result())?),
            }))
        }

        /// Returns the default result for the mock.
        pub fn default_result() -> VersionResult {
            VersionResult {
                current: VERSION.into(),
                supported_versions: vec![VERSION.into()],
            }
        }

        fn result(&self) -> Result<String> {
            match &self.result {
                Ok(s) => Ok(s.clone()),
                Err(e) => bail!("{}", e),
            }
        }
    }

    #[tokio::test]
    async fn find_binary_success() -> Result<()> {
        let plugin = PluginBuilder::default().binary("ls").build()?;
        assert!(plugin.find_binary("").is_ok());
        Ok(())
    }

    #[tokio::test]
    async fn find_binary_failure_not_found() -> Result<()> {
        let plugin = PluginBuilder::default().binary("").build()?;
        assert!(plugin.find_binary("").is_err());
        Ok(())
    }

    #[tokio::test]
    async fn version_success() -> Result<()> {
        let mut plugin = PluginBuilder::default().binary("").build()?;
        plugin.set_exec(ExecMock::boxed()?);

        let version = plugin.version().await?;

        assert_eq!(version.current(), VERSION);
        assert_eq!(version.supported_versions().len(), 1);
        Ok(())
    }

    #[tokio::test]
    async fn version_failure_output() -> Result<()> {
        let mut mock = ExecMock::boxed()?;
        let mut plugin = PluginBuilder::default().binary("").build()?;
        mock.result = Ok("wrong-output".into());
        plugin.set_exec(mock);

        assert!(plugin.version().await.is_err());
        Ok(())
    }

    #[tokio::test]
    async fn version_failure_exec() -> Result<()> {
        let mut plugin = PluginBuilder::default().binary("").build()?;
        let mut mock = ExecMock::boxed()?;
        mock.result = Err(format_err!(""));
        plugin.set_exec(mock);

        assert!(plugin.version().await.is_err());
        Ok(())
    }

    #[tokio::test]
    async fn add_success() -> Result<()> {
        let mut mock = ExecMock::boxed()?;
        let mut plugin = PluginBuilder::default().binary("ls").build()?;
        mock.result = Ok(r#"
        {
            "cniVersion": "0.3.1",
            "interfaces": [
                {
                    "name": "cni0",
                    "mac": "56:a8:20:5a:74:a4"
                },
                {
                    "name": "vethbd255036",
                    "mac": "f2:60:d0:99:d9:76"
                },
                {
                    "name": "eth0",
                    "mac": "72:6a:d7:f8:c4:84",
                    "sandbox": "/var/run/netns/test"
                }
            ],
            "ips": [
                {
                    "version": "4",
                    "interface": 2,
                    "address": "10.85.0.4/16",
                    "gateway": "10.85.0.1"
                },
                {
                    "version": "6",
                    "interface": 2,
                    "address": "1100:200::b9/24",
                    "gateway": "1100:200::1"
                }
            ],
            "routes": [
                {
                    "dst": "0.0.0.0/0"
                },
                {
                    "dst": "1100:200::1/24"
                }
            ],
            "dns": {}
        }"#
        .into());
        plugin.set_exec(mock);

        let result = plugin.add("", "", "", &[]).await?;

        assert_eq!(result.cni_version(), "0.3.1");
        assert_eq!(result.interfaces().len(), 3);
        assert_eq!(result.ips().len(), 2);
        assert_eq!(
            result
                .ips()
                .get(0)
                .context("no first addr")?
                .address()
                .prefix(),
            16
        );
        assert_eq!(
            result
                .ips()
                .get(1)
                .context("no second addr")?
                .address()
                .prefix(),
            24
        );
        assert_eq!(result.routes().len(), 2);
        Ok(())
    }

    #[tokio::test]
    async fn add_failure_error() -> Result<()> {
        let mut mock = ExecMock::boxed()?;
        let mut plugin = PluginBuilder::default().binary("ls").build()?;
        mock.result = Err(format_err!(r#"{ "code": 123, "msg": "error" }"#));
        plugin.set_exec(mock);

        assert!(plugin.add("", "", "", &[]).await.is_err());
        Ok(())
    }

    #[tokio::test]
    async fn add_failure_malformed_output() -> Result<()> {
        let mut mock = ExecMock::boxed()?;
        let mut plugin = PluginBuilder::default().binary("ls").build()?;
        mock.result = Ok("wrong".into());
        plugin.set_exec(mock);

        assert!(plugin.add("", "", "", &[]).await.is_err());
        Ok(())
    }

    #[tokio::test]
    async fn del_success() -> Result<()> {
        let mut mock = ExecMock::boxed()?;
        let mut plugin = PluginBuilder::default().binary("ls").build()?;
        mock.result = Ok("".into());
        plugin.set_exec(mock);

        plugin.del("", "", "", &[]).await
    }

    #[tokio::test]
    async fn del_failure_error() -> Result<()> {
        let mut mock = ExecMock::boxed()?;
        let mut plugin = PluginBuilder::default().binary("ls").build()?;
        mock.result = Err(format_err!(r#"{ "code": 123, "msg": "error" }"#));
        plugin.set_exec(mock);

        assert!(plugin.del("", "", "", &[]).await.is_err());
        Ok(())
    }
}
