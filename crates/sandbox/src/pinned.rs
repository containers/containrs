//! A pod sandbox implementation which does pin it's namespaces to file descriptors.

use super::{LinuxNamespaces, Pod};
use crate::error::{Result, SandboxError};
use crate::pinns::Arg;
use crate::{Pinns, SandboxContext};
use async_trait::async_trait;
use tokio::fs;
use uuid::Uuid;

#[derive(Default)]
pub struct PinnedSandbox {}

#[async_trait]
impl Pod for PinnedSandbox {
    async fn run(&mut self, context: &SandboxContext) -> Result<()> {
        let config = &context.config;

        Self::pin_namespaces(
            Uuid::new_v4().to_string(),
            config.pinns(),
            config.linux_namespaces(),
        )
        .await?;

        Ok(())
    }

    fn stop(&mut self, _: &SandboxContext) -> Result<()> {
        Ok(())
    }

    fn remove(&mut self, _: &SandboxContext) -> Result<()> {
        Ok(())
    }

    fn ready(&mut self, _: &SandboxContext) -> Result<bool> {
        Ok(false)
    }
}

impl PinnedSandbox {
    async fn pin_namespaces(
        pod_id: String,
        pinns: &Pinns,
        namespaces: &Option<LinuxNamespaces>,
    ) -> Result<()> {
        if let Some(ns) = namespaces {
            let mut args = Vec::new();
            if ns.contains(LinuxNamespaces::IPC) {
                args.push(Arg::Ipc);
            }

            if ns.contains(LinuxNamespaces::UTS) {
                args.push(Arg::Uts);
            }

            if ns.contains(LinuxNamespaces::NET) {
                args.push(Arg::Net);
            }

            if ns.contains(LinuxNamespaces::CGROUP) {
                args.push(Arg::Cgroup)
            }

            fs::create_dir_all(&pinns.pin_dir()).await?;

            args.push(Arg::Dir(pinns.pin_dir().clone()));
            args.push(Arg::FileName(pod_id));
            args.push(Arg::LogLevel(pinns.log_level()));

            let output = pinns.run(&args).await?;
            if !output.status.success() {
                return Err(SandboxError::Pinning(format!(
                    "failed to pin namespaces. Pinns exited with {}. Output: {}",
                    output.status,
                    String::from_utf8(output.stderr).unwrap()
                )));
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;
    use crate::pinns::PinnsBuilder;
    use anyhow::{Context, Result};
    use nix::mount::{umount2, MntFlags};
    use tempfile::TempDir;

    // pinned namespaces need to be cleaned up, otherwise the test
    // directory can not be deleted
    fn cleanup_pinned_dir(namespaces: &[PathBuf]) {
        for ns_path in namespaces {
            let _ = umount2(ns_path, MntFlags::MNT_DETACH);
        }
    }

    #[tokio::test]
    async fn pin_namespaces() -> Result<()> {
        let pod_id = Uuid::new_v4().to_string();
        let pin_dir = TempDir::new().context("create temp dir")?;
        let namespaces = Some(LinuxNamespaces::IPC | LinuxNamespaces::UTS | LinuxNamespaces::NET);
        let pinns = PinnsBuilder::default()
            .binary(which::which("pinns")?)
            .pin_dir(pin_dir.path())
            .build()
            .context("build pinns")?;

        PinnedSandbox::pin_namespaces(pod_id.clone(), &pinns, &namespaces)
            .await
            .context("pin namespaces")?;

        let pinned_ns: Vec<PathBuf> = ["ipcns", "utsns", "netns"]
            .iter()
            .map(|ns| pin_dir.path().join(ns).join(&pod_id))
            .collect();

        for ns in &pinned_ns {
            assert!(ns.exists());
        }

        cleanup_pinned_dir(&pinned_ns);
        Ok(())
    }
}
