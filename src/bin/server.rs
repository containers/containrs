use anyhow::Result;
use containrs::kubernetes::server::{Config, Server};
use std::process::exit;

#[tokio::main]
async fn main() -> Result<()> {
    // Parse CLI arguments
    let config = Config::default();

    // Spawn the server based on the configuration
    if let Err(e) = Server::new(config).start().await {
        // Collect all errors and chain them together. Do not use the logger
        // for printing here, because it could be possible that it fails before
        // initializing it.
        println!("Unable to run server: {:#}", e);
        exit(1);
    }

    Ok(())
}
