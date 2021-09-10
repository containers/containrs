use anyhow::{format_err, Context, Result};
use cbindgen::{Builder, Config};
use std::{env, path::PathBuf};

fn main() -> Result<()> {
    let bindgen_config = PathBuf::from(env::var("CARGO_MANIFEST_DIR")?)
        .join("..")
        .join("..")
        .join(".cbindgen.toml");

    Builder::new()
        .with_crate(env::var("CARGO_MANIFEST_DIR")?)
        .with_config(Config::from_file(bindgen_config).map_err(|e| format_err!(e))?)
        .generate()
        .context("generate bindings")?
        .write_to_file("src/ffi.h");

    Ok(())
}
