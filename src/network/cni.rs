//! A network implementation which does work with the Kubernetes Container Network Interface (CNI).

use anyhow::Result;
use derive_builder::Builder;
use getset::Getters;
use log::{debug, info};
use std::path::PathBuf;

#[derive(Builder, Default, Getters)]
#[builder(pattern = "owned", setter(into))]
/// The pod network implementation based on the Container Network Interface.
pub struct CNI {
    #[get]
    /// The network name of the default network to be used.
    default_network_name: Option<String>,

    #[get]
    /// The paths to configuration files.
    config_paths: Vec<PathBuf>,

    #[get]
    /// The paths to binary plugins.
    plugin_paths: Vec<PathBuf>,

    #[builder(default = "false")]
    #[get]
    /// Specifies if the network is ready or not
    ready: bool,
}

impl CNI {
    /// Initialize the CNI network
    pub fn initialize(&mut self) -> Result<()> {
        info!("Initializing CNI network");
        debug!("Default network name: {:?}", self.default_network_name());

        let path_bufs_to_string = |x: &[PathBuf]| -> String {
            x.iter()
                .map(|x| x.display().to_string())
                .collect::<Vec<_>>()
                .join(", ")
        };
        debug!(
            "Configuration paths: {}",
            path_bufs_to_string(self.config_paths())
        );
        debug!("Plugin paths: {}", path_bufs_to_string(self.plugin_paths()));

        self.ready = true;

        Ok(())
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    fn initialize_success() -> Result<()> {
        let mut cni = CNIBuilder::default()
            .default_network_name(Some("network".into()))
            .config_paths(["a", "b"].iter().map(PathBuf::from).collect::<Vec<_>>())
            .plugin_paths(["c", "d"].iter().map(PathBuf::from).collect::<Vec<_>>())
            .build()?;

        assert!(!cni.ready());
        assert!(cni.default_network_name.is_some());
        assert_eq!(cni.config_paths().len(), 2);
        assert_eq!(cni.plugin_paths().len(), 2);

        cni.initialize()?;
        assert!(cni.ready());

        Ok(())
    }
}
