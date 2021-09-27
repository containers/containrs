use thiserror::Error;

#[derive(Debug, Error)]
pub enum ServiceError {
    #[error("{0}")]
    Other(String),
}