//! A pod sandbox implementation which does pin it's namespaces to file descriptors.

use tokio::fs;
use uuid::Uuid;
use super::{LinuxNamespaces, Pod};
use crate::error::{Result, SandboxError};
use crate::pinns::Arg;
use crate::{Pinns, SandboxContext};
use async_trait::async_trait;

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

            let pin_dir = pinns.pin_dir().join(&pod_id);
            fs::create_dir_all(&pin_dir).await?;

            args.push(Arg::Dir(pin_dir));
            args.push(Arg::FileName(pod_id));
            args.push(Arg::LogLevel(pinns.log_level()));

            let output = pinns.run(&args).await?;
            if !output.status.success() {
                return Err(SandboxError::Pinning(format!(
                    "failed to pin namespaces. Pinns exited with {}",
                    output.status
                )));
            }
        }

        Ok(())
    }
}
