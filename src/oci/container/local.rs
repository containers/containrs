//! A local Command Line Interface based OCI runtime implementation. The most commonly known are
//! [runc][0] and [crun][1].
//!
//! [0]: https://github.com/opencontainers/runc
//! [1]: https://github.com/containers/crun

use crate::oci::{
    container::{Container, ContainerState, ContainerStats},
    spec::runtime::{LinuxResources, Spec},
};
use anyhow::Result;
use async_trait::async_trait;
use derive_builder::Builder;
use getset::Getters;
use serde::{Deserialize, Serialize};
use tokio::{process::Command, signal::unix::SignalKind};

#[derive(Debug, Default, Builder, Getters, Serialize, Deserialize)]
#[builder(default, pattern = "owned", setter(into, strip_option))]
/// A general OCI container implementation.
pub struct OCIContainer {
    #[get = "pub"]
    /// Unique identifier of the container.
    id: String,

    #[get = "pub"]
    /// OCI Runtime Specification of the container.
    spec: Spec,
}

#[async_trait]
impl Container for OCIContainer {
    /// Create a new container, which should be in the `Created` state afterwards.
    async fn create() -> Result<Self> {
        unimplemented!()
    }

    /// Execute the user defined process in a created container.
    async fn start(&mut self) -> Result<()> {
        unimplemented!()
    }

    /// Delete any resources held by the container often used with detached container.
    async fn delete(&mut self) -> Result<()> {
        unimplemented!()
    }

    /// Suspend all processes inside the container.
    async fn pause(&mut self) -> Result<()> {
        unimplemented!()
    }

    /// Resumes all processes that have been previously paused.
    async fn resume(&mut self) -> Result<()> {
        unimplemented!()
    }

    /// Send the specified signal to the container's init process.
    async fn kill(&mut self, _signal_kind: SignalKind) -> Result<()> {
        unimplemented!()
    }

    /// Update container resource constraints.
    async fn update(&mut self, _resources: &LinuxResources) -> Result<()> {
        unimplemented!()
    }

    /// Execute the provided process inside the container.
    async fn exec(&self, _command: &Command) -> Result<()> {
        unimplemented!()
    }

    /// Retrieve container resource statistics.
    async fn stats(&self) -> Result<ContainerStats> {
        unimplemented!()
    }

    /// Retrieve the state of a container.
    async fn state(&self) -> Result<ContainerState> {
        unimplemented!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn container_create() -> Result<()> {
        let container = OCIContainerBuilder::default().id("id").build()?;
        assert_eq!(container.id(), "id");
        assert_eq!(container.spec(), &Spec::default());
        Ok(())
    }
}
