use thiserror::Error;
use tokio::io;

pub type Result<T> = std::result::Result<T, SandboxError>;

#[derive(Error, Debug)]
pub enum SandboxError {
    #[error("uninitialized field")]
    Builder(#[from] derive_builder::UninitializedFieldError),
    #[error("{0}")]
    Pinning(String),
    #[error("IO")]
    IO(#[from] io::Error),
}
