//! Network types and implementations.

use anyhow::Result;
use async_trait::async_trait;
use derive_builder::Builder;
use sandbox::SandboxData;

pub mod cni;

#[derive(Builder)]
#[builder(pattern = "owned", setter(into))]
/// Network is the main structure for working with the Container Network Interface.
/// The implementation `T` can vary and is being defined in the `Pod` trait.
pub struct Network<T>
where
    T: Default,
{
    #[builder(default = "T::default()")]
    /// Trait implementation for the network.
    implementation: T,
}

#[async_trait]
/// Common network behavior trait
pub trait PodNetwork {
    /// Start a new network for the provided `SandboxData`.
    async fn start(&mut self, _: &SandboxData) -> Result<()> {
        Ok(())
    }

    /// Stop the network of the provided `SandboxData`.
    async fn stop(&mut self, _: &SandboxData) -> Result<()> {
        Ok(())
    }

    /// Cleanup the network implementation on server shutdown.
    async fn cleanup(&mut self) -> Result<()> {
        Ok(())
    }
}

impl<T> Network<T>
where
    T: Send + Default + PodNetwork,
{
    #[allow(dead_code)]
    /// Wrapper for the implementations `start` method.
    pub async fn start(&mut self, sandbox_data: &SandboxData) -> Result<()> {
        self.implementation.start(sandbox_data).await
    }

    #[allow(dead_code)]
    /// Wrapper for the implementations `stop` method.
    pub async fn stop(&mut self, sandbox_data: &SandboxData) -> Result<()> {
        self.implementation.stop(sandbox_data).await
    }

    /// Cleanup the network implementation on server shutdown.
    pub async fn cleanup(&mut self) -> Result<()> {
        self.implementation.cleanup().await
    }
}

#[cfg(test)]
pub mod tests {
    use std::collections::HashMap;

    use sandbox::{LinuxNamespaces, SandboxDataBuilder};

    use super::*;

    pub fn new_sandbox_data() -> Result<SandboxData> {
        let mut annotations: HashMap<String, String> = HashMap::new();
        annotations.insert("annotationkey1".into(), "annotationvalue1".into());

        Ok(SandboxDataBuilder::default()
            .id("uid")
            .name("name")
            .namespace("namespace")
            .attempt(1u32)
            .linux_namespaces(LinuxNamespaces::NET)
            .hostname("hostname")
            .log_directory("log_directory")
            .annotations(annotations)
            .build()?)
    }

    #[derive(Default)]
    struct Mock {
        start_called: bool,
        stop_called: bool,
    }

    #[async_trait]
    impl PodNetwork for Mock {
        async fn start(&mut self, _: &SandboxData) -> Result<()> {
            self.start_called = true;
            Ok(())
        }

        async fn stop(&mut self, _: &SandboxData) -> Result<()> {
            self.stop_called = true;
            Ok(())
        }
    }

    #[tokio::test]
    async fn create() -> Result<()> {
        let implementation = Mock::default();

        assert!(!implementation.start_called);
        assert!(!implementation.stop_called);

        let mut network = NetworkBuilder::<Mock>::default()
            .implementation(implementation)
            .build()?;

        let sandbox_data = new_sandbox_data()?;

        network.start(&sandbox_data).await?;
        assert!(network.implementation.start_called);

        network.stop(&sandbox_data).await?;
        assert!(network.implementation.stop_called);

        Ok(())
    }
}
