//! A CRI API service implementation.

use crate::storage::default_key_value_storage::DefaultKeyValueStorage;
use log::debug;
use std::fmt::Debug;
use tonic::{Request, Response, Status};

#[derive(Clone)]
/// The service implementation for the CRI API
pub struct CRIService {
    storage: DefaultKeyValueStorage,
}

impl CRIService {
    /// Create a new CRIService with the provided storage.
    pub fn new(storage: DefaultKeyValueStorage) -> Self {
        Self { storage }
    }

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
