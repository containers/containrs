use anyhow::{Context, Result};

fn main() -> Result<()> {
    tonic_build::configure()
        .out_dir("src/criapi")
        .compile(&["proto/criapi.proto"], &["proto"])
        .context("compile CRI protocol buffers")
}
