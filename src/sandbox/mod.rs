//! Basic Pod Sandbox types

pub mod pinned;

use anyhow::Result;
use derive_builder::Builder;
use getset::Getters;
use std::fmt;

#[derive(Builder)]
#[builder(pattern = "owned", setter(into))]
/// This is the main data structure for a Pod Sandbox. The implementation `T` can vary and is being
/// defined in the `Pod` trait. Responsibility of the `Sandbox` is to hold arbitrary necessary data
/// for the implementation and not modify it in any way.
pub struct Sandbox<T>
where
    T: Default,
{
    data: SandboxData,

    #[builder(default = "T::default()")]
    /// Trait implementation for creating the sandbox.
    implementation: T,
}

#[derive(Builder, Getters)]
#[builder(pattern = "owned", setter(into))]
/// SandboxData holds all the data which will be passed around to the `Pod` trait, too.
pub struct SandboxData {
    #[get = "pub"]
    /// The unique identifier.
    id: String,

    #[get = "pub"]
    /// Full name of the sandbox.
    name: String,

    #[get = "pub"]
    /// Namespace where the sandbox lives in.
    namespace: String,

    #[get = "pub"]
    /// Sandbox creation attempt. It only changes if the Kubernetes sandbox data changed or dies
    /// because of any error, not if the sandbox creation itself fails.
    attempt: u32,
}

pub trait Pod {
    /// Run a previously created sandbox.
    fn run(&mut self, _: &SandboxData) -> Result<()> {
        Ok(())
    }

    /// Stop a previously started sandbox.
    fn stop(&mut self, _: &SandboxData) -> Result<()> {
        Ok(())
    }

    /// Remove a stopped sandbox.
    fn remove(&mut self, _: &SandboxData) -> Result<()> {
        Ok(())
    }

    // Returns whether a sandbox is ready or not
    fn ready(&mut self, _: &SandboxData) -> Result<bool> {
        Ok(false)
    }
}

impl<T> Sandbox<T>
where
    T: Default + Pod,
{
    /// Retrieve the unique identifier for the sandbox
    pub fn id(&self) -> &str {
        &self.data.id
    }

    /// Wrapper for the implementations `run` method
    pub fn run(&mut self) -> Result<()> {
        self.implementation.run(&self.data)
    }

    #[allow(dead_code)]
    /// Wrapper for the implementations `stop` method
    pub fn stop(&mut self) -> Result<()> {
        self.implementation.stop(&self.data)
    }

    #[allow(dead_code)]
    /// Wrapper for the implementations `remove` method
    pub fn remove(&mut self) -> Result<()> {
        self.implementation.remove(&self.data)
    }

    #[allow(dead_code)]
    /// Wrapper for the implementations `remove` method
    pub fn ready(&mut self) -> Result<bool> {
        self.implementation.ready(&self.data)
    }
}

impl<T> fmt::Debug for Sandbox<T>
where
    T: Default,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Sandbox")
            .field("id", self.data.id())
            .field("name", self.data.name())
            .field("namespace", self.data.namespace())
            .field("attempt", self.data.attempt())
            .finish()
    }
}

impl<T> fmt::Display for Sandbox<T>
where
    T: Default,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} ({})", self.data.name(), self.data.id())
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use anyhow::format_err;

    #[derive(Default)]
    struct Mock {
        run_called: bool,
        stop_called: bool,
        remove_called: bool,
        ready: bool,
    }
    impl Pod for Mock {
        fn run(&mut self, _: &SandboxData) -> Result<()> {
            self.run_called = true;
            self.ready = true;
            Ok(())
        }
        fn stop(&mut self, _: &SandboxData) -> Result<()> {
            self.stop_called = true;
            self.ready = false;
            Ok(())
        }
        fn remove(&mut self, _: &SandboxData) -> Result<()> {
            self.remove_called = true;
            Ok(())
        }
        fn ready(&mut self, _: &SandboxData) -> Result<bool> {
            Ok(self.ready)
        }
    }

    #[test]
    fn create() -> Result<()> {
        let sandbox = SandboxBuilder::<Mock>::default()
            .data(
                SandboxDataBuilder::default()
                    .id("uid")
                    .name("name")
                    .namespace("namespace")
                    .attempt(1u32)
                    .build()
                    .map_err(|e| format_err!("build sandbox data: {}", e))?,
            )
            .build()
            .map_err(|e| format_err!("build sandbox: {}", e))?;

        assert_eq!(sandbox.id(), sandbox.data.id());

        let sandbox_display = format!("{}", sandbox);
        assert!(sandbox_display.contains(sandbox.data.id()));
        assert!(sandbox_display.contains(sandbox.data.name()));

        let sandbox_debug = format!("{:?}", sandbox);
        assert!(sandbox_debug.contains(sandbox.data.name()));
        assert!(sandbox_debug.contains(sandbox.data.namespace()));
        assert!(sandbox_debug.contains(sandbox.data.id()));
        assert!(sandbox_debug.contains(&sandbox.data.attempt().to_string()));
        Ok(())
    }

    #[test]
    fn create_custom_impl() -> Result<()> {
        let implementation = Mock::default();

        assert!(!implementation.run_called);
        assert!(!implementation.stop_called);
        assert!(!implementation.remove_called);

        let mut sandbox = SandboxBuilder::<Mock>::default()
            .data(
                SandboxDataBuilder::default()
                    .id("id")
                    .name("name")
                    .namespace("namespace")
                    .attempt(0u32)
                    .build()
                    .map_err(|e| format_err!("build sandbox data: {}", e))?,
            )
            .implementation(implementation)
            .build()
            .map_err(|e| format_err!("build sandbox: {}", e))?;

        assert!(!sandbox.ready().unwrap());
        sandbox.run()?;
        assert!(sandbox.implementation.run_called);
        assert!(sandbox.ready().unwrap());

        sandbox.stop()?;
        assert!(sandbox.implementation.stop_called);
        assert!(!sandbox.ready().unwrap());

        sandbox.remove()?;
        assert!(sandbox.implementation.remove_called);
        assert!(!sandbox.ready().unwrap());

        Ok(())
    }
}
