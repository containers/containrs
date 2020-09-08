use crate::{
    config::Config, criapi::image_service_server::ImageServiceServer,
    criapi::runtime_service_server::RuntimeServiceServer, image_service::MyImage,
    runtime_service::MyRuntime,
};
use anyhow::{Context, Result};
use clap::crate_name;
use log::info;
use std::env;
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

        let addr = "[::1]:50051".parse().context("parse server address")?;
        let rt = MyRuntime::default();
        let img = MyImage::default();

        info!("Runtime server listening on {}", addr);

        transport::Server::builder()
            .add_service(RuntimeServiceServer::new(rt))
            .add_service(ImageServiceServer::new(img))
            .serve(addr)
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
