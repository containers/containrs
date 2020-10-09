//! A pod sandbox implementation which does pin it's namespaces to file descriptors.

use crate::sandbox::Pod;
use crate::{
    capability::Capabilities,
    oci_spec::runtime::{
        LinuxCapabilities, LinuxCapabilitiesBuilder, Process, ProcessBuilder, Spec, SpecBuilder,
    },
    sandbox::SandboxData,
};
use anyhow::{Context, Result};
use getset::Getters;
use log::{debug, trace};

#[derive(Default, Getters)]
pub struct PinnedSandbox {
    #[get]
    runtime_spec: Spec,
}

impl Pod for PinnedSandbox {
    /// Run a new sandbox.
    fn run(&mut self, sandbox_data: &SandboxData) -> Result<()> {
        debug!("Running pod sandbox: {:?}", sandbox_data);

        // Build the OCI runtime specification
        let runtime_spec = self.build_runtime_spec()?;
        debug!(
            "Built OCI runtime spec for sandbox {}: {:?}",
            sandbox_data.id(),
            runtime_spec
        );

        // Update the sandbox state
        self.runtime_spec = runtime_spec;
        Ok(())
    }
}

impl PinnedSandbox {
    /// Build the runtime spec.
    pub fn build_runtime_spec(&self) -> Result<Spec> {
        trace!("Building OCI runtime spec");
        let spec = SpecBuilder::default()
            .process(self.build_runtime_spec_process()?)
            .build()
            .context("build OCI runtime spec")?;
        Ok(spec)
    }

    /// Build the runtime spec process.
    fn build_runtime_spec_process(&self) -> Result<Process> {
        trace!("Building OCI runtime spec process");
        Ok(ProcessBuilder::default()
            .capabilities(self.build_runtime_spec_capabilities()?)
            .build()
            .context("build process")?)
    }

    /// Build the runtime spec process Linux capabilities.
    fn build_runtime_spec_capabilities(&self) -> Result<LinuxCapabilities> {
        trace!("Building OCI runtime spec capabilities");
        let default_capabilities = Capabilities::default();
        Ok(LinuxCapabilitiesBuilder::default()
            .bounding(&default_capabilities)
            .effective(&default_capabilities)
            .inheritable(&default_capabilities)
            .permitted(&default_capabilities)
            .build()
            .context("build capabilities")?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sandbox::tests::new_sandbox_data;

    #[test]
    fn run_success() -> Result<()> {
        let sandbox_data = new_sandbox_data()?;
        let mut sandbox = PinnedSandbox::default();

        sandbox.run(&sandbox_data)?;

        let capabilities = sandbox
            .runtime_spec()
            .process()
            .as_ref()
            .context("no process")?
            .capabilities()
            .as_ref()
            .context("no capabilities")?;
        let default_capabilities: Vec<String> = Capabilities::default().into();
        assert_eq!(
            capabilities.bounding().as_ref().context("no boundings")?,
            &default_capabilities
        );
        assert_eq!(
            capabilities.effective().as_ref().context("no effective")?,
            &default_capabilities
        );
        assert_eq!(
            capabilities
                .inheritable()
                .as_ref()
                .context("no effective")?,
            &default_capabilities
        );
        assert_eq!(
            capabilities.permitted().as_ref().context("no effective")?,
            &default_capabilities
        );
        assert!(capabilities.ambient().is_none());

        Ok(())
    }
}
