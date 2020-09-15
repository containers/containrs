use crate::{
    config::Config,
    criapi::{
        image_service_server::ImageServiceServer, runtime_service_server::RuntimeServiceServer,
    },
    image_service::MyImage,
    runtime_service::MyRuntime,
    unix_stream,
};
use anyhow::{bail, Context, Result};
use clap::crate_name;
use futures_util::stream::TryStreamExt;
use log::{debug, info};
use std::env;
#[cfg(unix)]
use tokio::net::UnixListener;
use tokio::{
    fs,
    signal::unix::{signal, SignalKind},
};
use tonic::{transport, Request, Status};

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
            let sock_dir = sock_path.parent().context("get socket path directory")?;
            fs::create_dir_all(sock_dir)
                .await
                .with_context(|| format!("create socket dir {}", sock_dir.display()))?;
        }

        let mut uds = UnixListener::bind(self.config.sock_path())?;

        info!(
            "Runtime server listening on {}",
            self.config.sock_path().display()
        );

        let rt = MyRuntime::default();
        let img = MyImage::default();

        // Handle shutdown based on signals
        let mut shutdown_terminate = signal(SignalKind::terminate())?;
        let mut shutdown_interrupt = signal(SignalKind::interrupt())?;

        tokio::select! {
            res = transport::Server::builder()
                .add_service(RuntimeServiceServer::with_interceptor(rt, Self::intercept))
                .add_service(ImageServiceServer::with_interceptor(img, Self::intercept))
                .serve_with_incoming(uds.incoming().map_ok(unix_stream::UnixStream)) => {
                res.context("run GRPC server")?
            }
            _ = shutdown_interrupt.recv() => {
                info!("Got interrupt signal, shutting down server");
            }
            _ = shutdown_terminate.recv() => {
                info!("Got termination signal, shutting down server");
            }
        }

        self.cleanup()
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

    /// This function will get called on each inbound request, if a `Status`
    /// is returned, it will cancel the request and return that status to the
    /// client.
    fn intercept(req: Request<()>) -> std::result::Result<Request<()>, Status> {
        debug!("{:?}", req);
        Ok(req)
    }

    /// Cleanup the server and persist any data if necessary.
    fn cleanup(self) -> Result<()> {
        debug!("Cleaning up server");
        Ok(())
    }
}
