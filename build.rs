use anyhow::{format_err, Context, Result};
use cbindgen::{Builder, Config};
use std::{env, path::PathBuf};

const PROTO_FILE: &str = "src/kubernetes/cri/proto/criapi.proto";

fn main() -> Result<()> {
    tonic_build::configure()
        .out_dir("src/kubernetes/cri/api")
        .compile(
            &[PROTO_FILE],
            &[&PathBuf::from(PROTO_FILE)
                .parent()
                .context("no path parent")?
                .display()
                .to_string()],
        )
        .context("compile CRI protocol buffers")?;

    Builder::new()
        .with_crate(env::var("CARGO_MANIFEST_DIR")?)
        .with_config(Config::from_file(".cbindgen.toml").map_err(|e| format_err!(e))?)
        .generate()
        .context("generate bindings")?
        .write_to_file("src/ffi/ffi.h");

    Ok(())
}
