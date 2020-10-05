//! Network namespace helpers and structures.

use anyhow::{Context, Result};
use futures::executor;
use getset::Getters;
use log::trace;
use nix::sched::{setns, CloneFlags};
use std::{
    fs,
    future::Future,
    os::unix::io::{AsRawFd, RawFd},
    path::{Path, PathBuf},
};
use tokio::{fs::File, task};

#[derive(Debug, Getters)]
/// A basic network namespace abstraction.
pub struct Namespace {
    #[get]
    /// The current namespace as File.
    current: File,

    #[get]
    /// The target namespace as File.
    target: File,
}

impl Namespace {
    /// Create a new namespace.
    pub async fn new<P>(path: P) -> Result<Self>
    where
        P: AsRef<Path>,
    {
        let current = File::open(Self::current_thread_namespace_path())
            .await
            .context("open current thread namespace file")?;

        let target = File::open(&path)
            .await
            .context("open target namespace file")?;

        Ok(Self { current, target })
    }

    /// Run a future inside this network namespace
    pub async fn run<F>(&self, fun: F) -> Result<()>
    where
        F: Future<Output = Result<()>> + Send + 'static,
    {
        trace!(
            "Using file as target network namespace: {:?}",
            self.target()
        );
        let current_fd = self.current().as_raw_fd();
        let target_fd = self.target().as_raw_fd();

        task::spawn_blocking(move || {
            // Switch to the target namespace
            trace!("Switching to target namespace");
            Self::switch_namespace(target_fd)?;

            // Run the future
            let result = executor::block_on(fun).context("run namespace future");

            // Ensure that we will switch back to the original network namespace
            trace!("Switching back to host network namespace");
            Self::switch_namespace(current_fd)?;

            result
        })
        .await
        .context("spawn namespace thread")?
        .context("run in namespace thread")
    }

    /// Switch the network namespace to the provided raw file descriptor.
    fn switch_namespace(fd: RawFd) -> Result<()> {
        trace!(
            "Current thread network namespace: {}",
            Self::current_thread_namespace()?.display(),
        );

        setns(fd, CloneFlags::CLONE_NEWNET).context("switch to network namespace")?;

        trace!(
            "Switched to network namespace: {}",
            Self::current_thread_namespace()?.display(),
        );
        Ok(())
    }

    /// Returns the current threads network namespace identifier.
    pub fn current_thread_namespace() -> Result<PathBuf> {
        fs::read_link(Self::current_thread_namespace_path())
            .context("get current thread network namespace")
    }

    /// Retrieve the current network namespace path of the thread.
    pub fn current_thread_namespace_path() -> &'static str {
        "/proc/thread-self/ns/net"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[tokio::test]
    async fn new_success() -> Result<()> {
        let temp_file = NamedTempFile::new()?;
        Namespace::new(temp_file.path()).await?;
        Ok(())
    }

    #[tokio::test]
    async fn new_failure_not_existing() {
        assert!(Namespace::new("/path/does/not/exist").await.is_err());
    }
}
