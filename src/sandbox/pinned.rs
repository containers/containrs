//! A pod sandbox implementation which does pin its namespaces to file descriptors.

use crate::sandbox::{LinuxNamespaces, Pod, SandboxData};
use async_trait::async_trait;
use anyhow::{bail, Context, Result};
use tokio::process::Command;
use which::which;
use std::path::PathBuf;

#[derive(Default)]
pub struct PinnedSandbox {
    pinner_path: PathBuf,
    namespaces_dir: String,
    ready: bool,
}

impl PinnedSandbox {
    pub fn new(pinner: &str) -> Result<PinnedSandbox> {
        let path = which(pinner)
            .with_context(|| format!("failed to find {} in PATH", pinner))?;
        let ps = PinnedSandbox {
            pinner_path: path,
            // TODO: Make this configurable
            namespaces_dir: String::from("/var/run/cri"),
            ready: false,
        };
        Ok(ps)
    }
}

#[async_trait]
impl Pod for PinnedSandbox {
    async fn run(&mut self, sd: &SandboxData) -> Result<()> {
        let mut args = Vec::new();
        args.push("-d");
        args.push(&self.namespaces_dir);
        args.push("-f");
        args.push(&sd.id);
        let linux_namespaces = sd.linux_namespaces.context("linux namespaces not set")?;
        if linux_namespaces.contains(LinuxNamespaces::NET) {
            args.push("-n");
        }
        if linux_namespaces.contains(LinuxNamespaces::IPC) {
            args.push("-i");
        }
        if linux_namespaces.contains(LinuxNamespaces::UTS) {
            args.push("-u");
        }
        let output = Command::new(&self.pinner_path).args(&args[..]).output().await?;
        if !output.status.success() {
            bail!("Failed to pin namespaces for sandbox")
        }
        self.ready = true;
        Ok(())
    }

    fn stop(&mut self, _: &SandboxData) -> Result<()> {
        self.ready = false;
        Ok(())
    }

    fn remove(&mut self, _: &SandboxData) -> Result<()> {
        Ok(())
    }

    fn ready(&mut self, _: &SandboxData) -> Result<bool> {
        Ok(false)
    }
}
