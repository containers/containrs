//! OCI container implementations.

use crate::oci::spec::runtime::LinuxResources;
use anyhow::Result;
use async_trait::async_trait;
use serde::{de::DeserializeOwned, Serialize};
use strum::{AsRefStr, Display, EnumString, IntoStaticStr};
use tokio::{process::Command, signal::unix::SignalKind};

pub mod local;

#[async_trait]
/// Container is the trait for implementing possible interactions with an OCI compatible container.
pub trait Container
where
    Self: Sized + Send + Sync + Serialize + DeserializeOwned,
{
    /// Create a new container, which should be in the `Created` state afterwards.
    async fn create() -> Result<Self>;

    /// Execute the user defined process in a created container.
    async fn start(&mut self) -> Result<()>;

    /// Delete any resources held by the container often used with detached container.
    async fn delete(&mut self) -> Result<()>;

    /// Suspend all processes inside the container.
    async fn pause(&mut self) -> Result<()>;

    /// Resumes all processes that have been previously paused.
    async fn resume(&mut self) -> Result<()>;

    /// Send the specified signal to the container's init process.
    async fn kill(&mut self, signal_kind: SignalKind) -> Result<()>;

    /// Update container resource constraints.
    async fn update(&mut self, resources: &LinuxResources) -> Result<()>;

    /// Execute the provided process inside the container.
    async fn exec(&self, command: &Command) -> Result<()>;

    /// Retrieve container resource statistics.
    async fn stats(&self) -> Result<ContainerStats>;

    /// Retrieve the state of a container.
    async fn state(&self) -> Result<ContainerState>;
}

#[derive(Debug, Default)]
/// Container resource statistics.
pub struct ContainerStats;

#[derive(AsRefStr, Clone, Copy, Debug, Display, EnumString, Eq, Hash, IntoStaticStr, PartialEq)]
#[strum(serialize_all = "snake_case")]
/// Possible container states.
pub enum ContainerState {
    /// The container has been created (default state).
    Created,

    /// The container is running, usually after calling its `start()` trait method.
    Started,

    /// The container is paused, usually after calling its `pause()` trait method.
    Paused,

    /// The container is stopped, usually after calling its `kill()` trait method.
    Killed,
}
