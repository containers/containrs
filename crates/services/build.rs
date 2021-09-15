use anyhow::{Context, Result};
use std::path::PathBuf;

const PROTO_FILE: &str = "src/cri/proto/criapi.proto";

fn main() -> Result<()> {
    tonic_build::configure()
        .out_dir("src/cri/api")
        .compile(
            &[PROTO_FILE],
            &[&PathBuf::from(PROTO_FILE)
                .parent()
                .context("no path parent")?
                .display()
                .to_string()],
        )
        .context("compile CRI protocol buffers")?;
    Ok(())
}
