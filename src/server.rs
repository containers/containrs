use crate::{
    config::Config, criapi::image_service_server::ImageServiceServer,
    criapi::runtime_service_server::RuntimeServiceServer, image_service::MyImage,
    runtime_service::MyRuntime,
    unix_stream,
};
use anyhow::{anyhow, Context, Result};
use clap::crate_name;
use log::info;
use std::env;
use std::path::Path;
#[cfg(unix)]
use tokio::net::UnixListener;
use futures_util::stream::TryStreamExt;
use tokio::fs;
use tonic::transport;

/// Server is the main instance to run the Container Runtime Interface
pub struct Server {
    config: Config,
}

impl Server {
    /// Create a new server instance
    pub fn new(config: Config) -> Self {
        Server { config }
    }

    /// Start a new server with its default values
    pub async fn start(self) -> Result<()> {
        self.set_logging_verbosity()
            .context("set logging verbosity")?;

        let rt = MyRuntime::default();
        let img = MyImage::open(&self.config.data_dir()).await?;

        let sock_path = Path::new(self.config.sock_path());
        if !sock_path.is_absolute() {
            return Err(anyhow!("specified socket path {} is not absolute", sock_path.display()));
        }
        if sock_path.exists() {
            fs::remove_file(sock_path).await?;
        } else {
            fs::create_dir_all(sock_path.parent().context("get socket path directory")?).await?;
        }

        let mut uds = UnixListener::bind(&self.config.sock_path())?;

        info!("Runtime server listening on {}", self.config.sock_path());

        transport::Server::builder()
            .add_service(RuntimeServiceServer::new(rt))
            .add_service(ImageServiceServer::new(img))
            .serve_with_incoming(uds.incoming().map_ok(unix_stream::UnixStream))
            .await
            .context("serve GRPC")
    }

    /// Initialize the logger and set the verbosity to the provided level.
    fn set_logging_verbosity(&self) -> Result<()> {
        // Set the logging verbosity via the env
        env::set_var(
            "RUST_LOG",
            format!("{}={}", crate_name!(), self.config.log_level()),
        );

        // Initialize the logger
        env_logger::try_init().context("init env logger")
    }
}
