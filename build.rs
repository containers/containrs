use anyhow::{Context, Result};
use tonic_build::compile_protos;

fn main() -> Result<()> {
    compile_protos("proto/criapi.proto").context("compile CRI protocol buffers")
}
