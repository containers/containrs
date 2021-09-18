//! Basic Pod Sandbox types

pub mod error;
pub mod pinned;
pub mod pinns;

use crate::error::{Result, SandboxError};
use async_trait::async_trait;
use bitflags::bitflags;
use common::Namespace;
use derive_builder::Builder;
use getset::{CopyGetters, Getters, Setters};
use pinns::Pinns;
use std::{collections::HashMap, fmt, path::PathBuf};

#[derive(Builder)]
#[builder(pattern = "owned", setter(into), build_fn(error = "SandboxError"))]
/// This is the main data structure for a Pod Sandbox. The implementation `T` can vary and is being
/// defined in the `Pod` trait. Responsibility of the `Sandbox` is to hold arbitrary necessary data
/// for the implementation and not modify it in any way.
pub struct Sandbox<T>
where
    T: Default,
{
    context: SandboxContext,

    #[builder(default = "T::default()")]
    /// Trait implementation for creating the sandbox.
    implementation: T,
}

bitflags! {
    pub struct LinuxNamespaces: u32 {
        const MOUNT = 0b00000001;
        const CGROUP = 0b00000010;
        const UTS = 0b000000100;
        const IPC = 0b00001000;
        const USER = 0b00010000;
        const PID = 0b000100000;
        const NET = 0b001000000;
    }
}

#[derive(Builder, Clone, Debug, Getters, CopyGetters)]
#[builder(
    pattern = "owned",
    setter(into, strip_option),
    build_fn(error = "SandboxError")
)]
/// SandboxData holds all the data which will be passed around to the `Pod` trait, too.
pub struct SandboxConfig {
    #[get = "pub"]
    /// The unique identifier.
    id: String,

    #[get = "pub"]
    /// Full name of the sandbox.
    name: String,

    #[get = "pub"]
    /// Namespace where the sandbox lives in.
    namespace: String,

    #[get_copy = "pub"]
    /// Sandbox creation attempt. It only changes if the Kubernetes sandbox config changed or dies
    /// because of any error, not if the sandbox creation itself fails.
    attempt: u32,

    #[get = "pub"]
    /// Linux namespaces held by the Sandbox.
    linux_namespaces: Option<LinuxNamespaces>,

    #[get = "pub"]
    /// Hostname of the sandbox.
    hostname: String,

    #[get = "pub"]
    // Path to the directory on the host in which container log files are stored.
    log_directory: PathBuf,

    #[get = "pub"]
    // Arbitrary metadata of the sandbox.
    annotations: HashMap<String, String>,

    #[get = "pub"]
    #[builder(default = "None")]
    // Path to the network namespace.
    network_namespace_path: Option<PathBuf>,

    // Options for pinning namespaces
    #[get = "pub"]
    pinns: Pinns,
}

#[derive(Clone, Debug, Getters, Setters)]
pub struct SandboxState {
    // User namespace of the sandbox
    #[getset(get = "pub", set = "pub")]
    user_ns: Option<Namespace>,
    // IPC namespace of the sandbox
    #[getset(get = "pub", set = "pub")]
    ipc_ns: Option<Namespace>,
    // UTS namespace of the sandbox
    #[getset(get = "pub", set = "pub")]
    uts_ns: Option<Namespace>,
    // Network namespace of the sandbox
    #[getset(get = "pub", set = "pub")]
    net_ns: Option<Namespace>,
}

impl Default for SandboxState {
    fn default() -> Self {
        Self {
            user_ns: Default::default(),
            ipc_ns: Default::default(),
            uts_ns: Default::default(),
            net_ns: Default::default(),
        }
    }
}

#[derive(Builder, Debug, Getters)]
#[builder(pattern = "owned", setter(into), build_fn(error = "SandboxError"))]
pub struct SandboxContext {
    #[get = "pub"]
    config: SandboxConfig,
    #[getset(get_mut = "pub")]
    #[builder(default)]
    state: SandboxState,
}

#[async_trait]
pub trait Pod {
    /// Run a previously created sandbox.
    async fn run(&mut self, _: &SandboxContext) -> Result<()> {
        Ok(())
    }

    /// Stop a previously started sandbox.
    fn stop(&mut self, _: &SandboxContext) -> Result<()> {
        Ok(())
    }

    /// Remove a stopped sandbox.
    fn remove(&mut self, _: &SandboxContext) -> Result<()> {
        Ok(())
    }

    // Returns whether a sandbox is ready or not. A sandbox should be `ready()` if running, which
    // means that a previous call to `run()` was successful and it has not been neither `stopped()`
    // nor already `removed()`.
    fn ready(&mut self, _: &SandboxContext) -> Result<bool> {
        Ok(false)
    }
}

impl<T> Sandbox<T>
where
    T: Default + Pod + Send,
{
    /// Retrieve the unique identifier for the sandbox
    pub fn id(&self) -> &str {
        &self.context.config.id
    }

    /// Wrapper for the implementations `run` method
    pub async fn run(&mut self) -> Result<()> {
        self.implementation.run(&self.context).await
    }

    #[allow(dead_code)]
    /// Wrapper for the implementations `stop` method
    pub fn stop(&mut self) -> Result<()> {
        self.implementation.stop(&self.context)
    }

    #[allow(dead_code)]
    /// Wrapper for the implementations `remove` method
    pub fn remove(&mut self) -> Result<()> {
        self.implementation.remove(&self.context)
    }

    #[allow(dead_code)]
    /// Wrapper for the implementations `ready` method
    pub fn ready(&mut self) -> Result<bool> {
        self.implementation.ready(&self.context)
    }
}

impl<T> fmt::Debug for Sandbox<T>
where
    T: Default,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let config = &self.context.config;

        f.debug_struct("Sandbox")
            .field("id", config.id())
            .field("name", config.name())
            .field("namespace", config.namespace())
            .field("attempt", &config.attempt())
            .field("linux_namespaces", config.linux_namespaces())
            .field("hostname", config.hostname())
            .field("log_directory", config.log_directory())
            .field("annotations", config.annotations())
            .field("network_namespace_path", config.network_namespace_path())
            .finish()
    }
}

impl<T> fmt::Display for Sandbox<T>
where
    T: Default,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let config = &self.context.config;
        write!(f, "{} ({})", config.name(), config.id())
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    pub fn new_sandbox_data() -> Result<SandboxConfig> {
        let mut annotations: HashMap<String, String> = HashMap::new();
        annotations.insert("annotationkey1".into(), "annotationvalue1".into());

        Ok(SandboxConfigBuilder::default()
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
        run_called: bool,
        stop_called: bool,
        remove_called: bool,
        ready: bool,
    }

    #[async_trait]
    impl Pod for Mock {
        async fn run(&mut self, _: &SandboxContext) -> Result<()> {
            self.run_called = true;
            self.ready = true;
            Ok(())
        }
        fn stop(&mut self, _: &SandboxContext) -> Result<()> {
            self.stop_called = true;
            self.ready = false;
            Ok(())
        }
        fn remove(&mut self, _: &SandboxContext) -> Result<()> {
            self.remove_called = true;
            Ok(())
        }
        fn ready(&mut self, _: &SandboxContext) -> Result<bool> {
            Ok(self.ready)
        }
    }

    #[test]
    fn create() -> Result<()> {
        let config = new_sandbox_data()?;

        let context = SandboxContextBuilder::default()
            .config(config.clone())
            .build()?;

        let sandbox = SandboxBuilder::<Mock>::default().context(context).build()?;

        assert_eq!(sandbox.id(), config.id());

        let sandbox_display = format!("{}", sandbox);
        assert!(sandbox_display.contains(config.id()));
        assert!(sandbox_display.contains(config.name()));

        let sandbox_debug = format!("{:?}", sandbox);
        assert!(sandbox_debug.contains(config.name()));
        assert!(sandbox_debug.contains(config.namespace()));
        assert!(sandbox_debug.contains(config.id()));
        assert!(sandbox_debug.contains(&config.attempt().to_string()));
        assert!(sandbox_debug.contains(config.hostname()));

        let log_dir = config.log_directory().display().to_string();
        assert!(sandbox_debug.contains(&log_dir));

        for (key, val) in config.annotations.iter() {
            assert!(sandbox_debug.contains(key));
            assert!(sandbox_debug.contains(val));
        }

        Ok(())
    }

    #[tokio::test]
    async fn create_custom_impl() -> Result<()> {
        let implementation = Mock::default();
        let context = SandboxContextBuilder::default()
            .config(new_sandbox_data()?)
            .build()?;

        assert!(!implementation.run_called);
        assert!(!implementation.stop_called);
        assert!(!implementation.remove_called);

        let mut sandbox = SandboxBuilder::<Mock>::default()
            .context(context)
            .implementation(implementation)
            .build()?;

        assert!(!sandbox.ready()?);
        sandbox.run().await?;
        assert!(sandbox.implementation.run_called);
        assert!(sandbox.ready()?);

        sandbox.stop()?;
        assert!(sandbox.implementation.stop_called);
        assert!(!sandbox.ready()?);

        sandbox.remove()?;
        assert!(sandbox.implementation.remove_called);
        assert!(!sandbox.ready()?);

        Ok(())
    }
}
