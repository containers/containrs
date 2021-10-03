use oci_spec::OciSpecError;
use std::io;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ServiceError {
    #[error("{0}")]
    Other(String),
    #[error("{0}")]
    IO(#[from] io::Error),
    #[error("{0}")]
    Spec(#[from] OciSpecError),
}
