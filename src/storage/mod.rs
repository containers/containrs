//! Basic storage types

pub mod default_key_value_storage;

use anyhow::Result;
use serde::{de::DeserializeOwned, Serialize};
use std::{convert::AsRef, path::Path};

/// The data storage trait which defines the methods a storage implementation should fulfill.
pub trait KeyValueStorage {
    /// Load the storage from the provided path.
    fn open(path: &Path) -> Result<Self>
    where
        Self: Sized;

    /// Get an arbitrary item from the storage.
    fn get<K, V>(&self, key: K) -> Result<Option<V>>
    where
        K: AsRef<[u8]>,
        V: DeserializeOwned;

    /// Insert an item into the storage.
    fn insert<K, V>(&mut self, key: K, value: V) -> Result<()>
    where
        K: AsRef<[u8]>,
        V: Serialize;

    /// Remove an item from the storage.
    fn remove<K>(&mut self, key: K) -> Result<()>
    where
        K: AsRef<[u8]>;

    /// Save the storage to disk so that it is safe to stop the application.
    fn persist(&mut self) -> Result<()>;
}
