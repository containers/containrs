//! Container Runtime Interface server implementation
use crate::{
    kubernetes::cri::{
        api::{
            image_service_server::ImageServiceServer, runtime_service_server::RuntimeServiceServer,
        },
        cri_service::CRIServiceBuilder,
    },
    network::{
        cni::{CNIBuilder, CNI},
        Network, NetworkBuilder,
    },
    storage::{default_key_value_storage::DefaultKeyValueStorage, KeyValueStorage},
    unix_stream::UnixStream,
};
use anyhow::{bail, Context, Result};
use clap::crate_name;
pub use config::{Config, LogScope};
use env_logger::fmt::Color;
use futures::TryFutureExt;
use log::{debug, info, trace, LevelFilter};
use std::{env, io::Write};
#[cfg(unix)]
use tokio::net::UnixListener;
use tokio::{
    fs,
    signal::unix::{signal, SignalKind},
};
use tonic::transport;

mod config;

/// Server is the main instance to run the Container Runtime Interface
pub struct Server {
    config: Config,
}

impl Server {
    /// Create a new server instance
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    /// Start a new server with its default values
    pub async fn start(self) -> Result<()> {
        self.set_logging_verbosity()
            .context("set logging verbosity")?;

        // Setup the storage and pass it to the service
        let storage =
            DefaultKeyValueStorage::open(&self.config.storage_path().join("cri-service"))?;
        let cri_service = CRIServiceBuilder::default()
            .storage(storage.clone())
            .build()?;

        let network = self.initialize_network().await.context("init network")?;

        // Build a new socket from the config
        let uds = self.unix_domain_listener().await?;

        // Handle shutdown based on signals
        let mut shutdown_terminate = signal(SignalKind::terminate())?;
        let mut shutdown_interrupt = signal(SignalKind::interrupt())?;

        info!(
            "Runtime server listening on {}",
            self.config.sock_path().display()
        );

        let incoming = async_stream::stream! {
            while let item = uds.accept().map_ok(|(st, _)| UnixStream(st)).await {
                yield item;
            }
        };

        tokio::select! {
            res = transport::Server::builder()
                .add_service(RuntimeServiceServer::new(cri_service.clone()))
                .add_service(ImageServiceServer::new(cri_service))
                .serve_with_incoming(incoming) => {
                res.context("run GRPC server")?
            }
            _ = shutdown_interrupt.recv() => {
                info!("Got interrupt signal, shutting down server");
            }
            _ = shutdown_terminate.recv() => {
                info!("Got termination signal, shutting down server");
            }
        }

        self.cleanup(storage, network).await
    }

    /// Create a new UnixListener from the configs socket path.
    async fn unix_domain_listener(&self) -> Result<UnixListener> {
        let sock_path = self.config.sock_path();
        if !sock_path.is_absolute() {
            bail!(
                "specified socket path {} is not absolute",
                sock_path.display()
            )
        }
        if sock_path.exists() {
            fs::remove_file(sock_path)
                .await
                .with_context(|| format!("unable to remove socket file {}", sock_path.display()))?;
        } else {
            let sock_dir = sock_path
                .parent()
                .context("unable to get socket path directory")?;
            fs::create_dir_all(sock_dir)
                .await
                .with_context(|| format!("unable to create socket dir {}", sock_dir.display()))?;
        }

        Ok(UnixListener::bind(sock_path).context("unable to bind socket from path")?)
    }

    /// Initialize the logger and set the verbosity to the provided level.
    fn set_logging_verbosity(&self) -> Result<()> {
        // Set the logging verbosity via the env
        let level = if self.config.log_scope() == LogScope::Global {
            self.config.log_level().to_string()
        } else {
            format!("{}={}", crate_name!(), self.config.log_level())
        };
        env::set_var("RUST_LOG", level);

        // Initialize the logger with the format:
        // [YYYY-MM-DDTHH:MM:SS:MMMZ LEVEL crate::module file:LINE] MSG…
        // The file and line will be only printed when running with debug or trace level.
        let log_level = self.config.log_level();
        env_logger::builder()
            .format(move |buf, r| {
                let mut style = buf.style();
                style.set_color(Color::Black).set_intense(true);
                writeln!(
                    buf,
                    "{}{} {:<5} {}{}{} {}",
                    style.value("["),
                    buf.timestamp_millis(),
                    buf.default_styled_level(r.level()),
                    r.target(),
                    match (log_level >= LevelFilter::Debug, r.file(), r.line()) {
                        (true, Some(file), Some(line)) => format!(" {}:{}", file, line),
                        _ => "".into(),
                    },
                    style.value("]"),
                    r.args()
                )
            })
            .try_init()
            .context("init env logger")
    }

    /// Create a new network and initialize it from the internal configuration.
    async fn initialize_network(&self) -> Result<Network<CNI>> {
        let mut cni_network = CNIBuilder::default()
            .default_network_name(self.config.cni_default_network().clone())
            .config_paths(self.config.cni_config_paths().clone())
            .plugin_paths(self.config.cni_plugin_paths())
            .storage_path(self.config.storage_path().join("cni"))
            .build()
            .context("build CNI network data")?;

        cni_network
            .initialize()
            .await
            .context("initialize CNI network")?;

        let network = NetworkBuilder::<CNI>::default()
            .implementation(cni_network)
            .build()
            .context("build CNI network")?;

        Ok(network)
    }

    /// Cleanup the server and persist any data if necessary.
    async fn cleanup(
        self,
        mut storage: DefaultKeyValueStorage,
        mut network: Network<CNI>,
    ) -> Result<()> {
        debug!("Cleaning up server");

        trace!("Persisting storage");
        storage.persist().context("persist storage")?;

        trace!("Removing socket path");
        std::fs::remove_file(self.config.sock_path()).with_context(|| {
            format!(
                "unable to remove socket path {}",
                self.config.sock_path().display()
            )
        })?;

        trace!("Stopping network");
        network.cleanup().await.context("clean up network")?;

        trace!("Server shut down");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kubernetes::server::config::ConfigBuilder;
    use tempfile::{tempdir, NamedTempFile};

    #[tokio::test]
    async fn unix_domain_listener_success() -> Result<()> {
        let sock_path = &tempdir()?.path().join("test.sock");
        let config = ConfigBuilder::default().sock_path(sock_path).build()?;
        let sut = Server::new(config);

        assert!(!sock_path.exists());
        sut.unix_domain_listener().await?;
        assert!(sock_path.exists());

        Ok(())
    }

    #[tokio::test]
    async fn unix_domain_listener_success_exists() -> Result<()> {
        let sock_path = NamedTempFile::new()?;
        let config = ConfigBuilder::default()
            .sock_path(sock_path.path())
            .build()?;
        let sut = Server::new(config);

        assert!(sock_path.path().exists());
        sut.unix_domain_listener().await?;
        assert!(sock_path.path().exists());

        Ok(())
    }

    #[tokio::test]
    async fn unix_domain_listener_fail_not_absolute() -> Result<()> {
        let config = ConfigBuilder::default()
            .sock_path("not/absolute/path")
            .build()?;
        let sut = Server::new(config);

        assert!(sut.unix_domain_listener().await.is_err());

        Ok(())
    }

    #[tokio::test]
    async fn initialize_network_success() -> Result<()> {
        let config = ConfigBuilder::default()
            .storage_path(tempdir()?.path())
            .build()?;
        let sut = Server::new(config);
        sut.initialize_network().await?;
        Ok(())
    }

    #[tokio::test]
    async fn initialize_network_wrong_storage_path() -> Result<()> {
        let config = ConfigBuilder::default()
            .storage_path("/proc/storage")
            .build()?;
        let sut = Server::new(config);
        assert!(sut.initialize_network().await.is_err());
        Ok(())
    }
}
