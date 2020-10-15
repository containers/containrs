//! A CRI API service implementation.

use crate::storage::default_key_value_storage::DefaultKeyValueStorage;
use anyhow::Result;
use derive_builder::Builder;
use log::debug;
use std::fmt::{Debug, Display};
use tonic::{Request, Response, Status};

#[derive(Clone, Builder)]
#[builder(pattern = "owned", setter(into))]
/// The service implementation for the CRI API
pub struct CRIService {
    /// Storage used by the service.
    storage: DefaultKeyValueStorage,
}

impl CRIService {
    /// Debug log a request.
    pub fn debug_request<T>(&self, request: &Request<T>)
    where
        T: Debug,
    {
        debug!("{:?}", request.get_ref());
    }

    /// Debug log a response.
    pub fn debug_response<T>(&self, response: &Result<Response<T>, Status>)
    where
        T: Debug,
    {
        debug!("{:?}", response.as_ref().map(|x| x.get_ref()));
    }
}

/// Option to Status transformer for less verbose request unpacking.
pub trait OptionStatus<T> {
    /// Maps the self type to an invalid argument status containing the provided `msg`.
    fn ok_or_invalid(self, msg: impl Into<String>) -> Result<T, Status>
    where
        Self: Sized,
    {
        self.ok_or_else(|| Status::invalid_argument(msg))
    }

    /// Transforms the `OptionStatus<T>` into a [`Result<T, E>`], mapping [`Some(v)`] to
    /// [`Ok(v)`] and [`None`] to [`Err(err())`].
    ///
    /// [`Result<T, E>`]: Result
    /// [`Ok(v)`]: Ok
    /// [`Err(err())`]: Err
    /// [`Some(v)`]: Some
    fn ok_or_else<E, F>(self, err: F) -> Result<T, E>
    where
        F: FnOnce() -> E;
}

impl<T> OptionStatus<T> for Option<T> {
    fn ok_or_else<E, F>(self, err: F) -> Result<T, E>
    where
        F: FnOnce() -> E,
    {
        self.ok_or_else(err)
    }
}

/// Result to Status transformer for less verbose request unpacking.
pub trait ResultStatus<T, E>
where
    E: Display,
{
    /// Maps the self type to an internal error status containing the provided `msg`.
    fn map_internal(self, msg: impl Into<String> + Display) -> Result<T, Status>
    where
        Self: Sized,
    {
        self.map_err(|e| Status::internal(format!("{}: {}", msg, e)))
    }

    /// Maps a `ResultStatus<T, E>` to `Result<T, F>` by applying a function to a
    /// contained [`Err`] value, leaving an [`Ok`] value untouched.
    ///
    /// This function can be used to pass through a successful result while handling
    /// an error.
    fn map_err<F, O>(self, op: O) -> Result<T, F>
    where
        O: FnOnce(E) -> F;
}

impl<T, E> ResultStatus<T, E> for Result<T, E>
where
    E: Display,
{
    fn map_err<F, O>(self, op: O) -> Result<T, F>
    where
        O: FnOnce(E) -> F,
    {
        self.map_err(op)
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::storage::KeyValueStorage;
    use anyhow::Result;
    use tempfile::TempDir;

    pub fn new_cri_service() -> Result<CRIService> {
        let dir = TempDir::new()?;
        Ok(CRIService {
            storage: DefaultKeyValueStorage::open(dir.path())?,
        })
    }
}
