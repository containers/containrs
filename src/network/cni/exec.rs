//! CNI plugin interaction via command execution

use anyhow::{bail, Context, Result};
use async_trait::async_trait;
use derive_builder::Builder;
use getset::Getters;
use std::{collections::HashMap, path::Path};
use tokio::process::Command;

#[async_trait]
/// The CNI command execution trait.
pub trait Exec {
    /// Run a command and return the output as result.
    async fn run(&self, binary: &Path, args: &Args) -> Result<String>;
}

#[derive(Clone, Debug, Default)]
/// DefaultExec is a wrapper which can be used to execute CNI plugins in a standard way.
pub struct DefaultExec;

#[async_trait]
impl Exec for DefaultExec {
    /// Execute a CNI plugin binary with the internally set args.
    async fn run(&self, binary: &Path, args: &Args) -> Result<String> {
        let output = Command::new(binary).envs(args.envs()).output().await?;

        if !output.status.success() {
            bail!("command executed with failing error code")
        }

        Ok(String::from_utf8(output.stdout).context("cannot convert output to string")?)
    }
}

#[derive(Clone, Debug, Default, Builder, Getters)]
#[builder(default, pattern = "owned", setter(into))]
// CNI arguments abstraction
pub struct Args {
    #[get]
    /// CNI command.
    command: String,

    #[get]
    /// CNI Container ID.
    container_id: String,

    #[get]
    /// Network Namespace to be used.
    network_namespace: String,

    #[get]
    /// Additional plugin arguments.
    plugin_args: Vec<String>,

    #[get]
    /// The interface name.
    interface_name: String,

    #[get]
    /// Additional CNI $PATH.
    path: String,
}

impl Args {
    /// Returns a HashMap for passing them as environment variables to the CNI plugin.
    fn envs(&self) -> HashMap<String, String> {
        let mut env = HashMap::new();
        env.insert("CNI_COMMAND".into(), self.command().clone());
        env.insert("CNI_CONTAINERID".into(), self.container_id().clone());
        env.insert("CNI_NETNS".into(), self.network_namespace().clone());
        env.insert("CNI_ARGS".into(), self.plugin_args().join(";"));
        env.insert("CNI_IFNAME".into(), self.interface_name().clone());
        env.insert("CNI_PATH".into(), self.path().clone());
        env
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[tokio::test]
    async fn exec_success() -> Result<()> {
        let binary = which::which("ls")?;
        let output = DefaultExec
            .run(&binary, &ArgsBuilder::default().build()?)
            .await?;
        assert!(output.contains("Cargo.toml"));
        Ok(())
    }

    #[tokio::test]
    async fn exec_failure() -> Result<()> {
        let binary = PathBuf::from("/should/not/exist");
        let res = DefaultExec
            .run(&binary, &ArgsBuilder::default().build()?)
            .await;
        assert!(res.is_err());
        Ok(())
    }
}
