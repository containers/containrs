use crate::KeyValueStorage;
use anyhow::Result;
use getset::{Getters, MutGetters};
use std::collections::HashMap;

#[derive(Debug, Clone, Getters, MutGetters)]
pub struct MemoryKeyValueStorage {
    #[getset(get, get_mut)]
    db: HashMap<Vec<u8>, Vec<u8>>,
}

impl Default for MemoryKeyValueStorage {
    fn default() -> Self {
        Self { db: HashMap::new() }
    }
}

impl KeyValueStorage for MemoryKeyValueStorage {
    fn open<P>(_: P) -> Result<Self>
    where
        Self: Sized,
        P: AsRef<std::path::Path>,
    {
        Ok(Self::default())
    }

    fn get<K, V>(&self, key: K) -> Result<Option<V>>
    where
        K: AsRef<[u8]>,
        V: serde::de::DeserializeOwned,
    {
        match self.db().get(key.as_ref()) {
            None => Ok(None),
            Some(value) => Ok(Some(rmp_serde::from_slice(value)?)),
        }
    }

    fn insert<K, V>(&mut self, key: K, value: V) -> Result<()>
    where
        K: AsRef<[u8]>,
        V: serde::Serialize,
    {
        self.db_mut()
            .insert(key.as_ref().to_vec(), rmp_serde::to_vec(&value)?);
        Ok(())
    }

    fn remove<K>(&mut self, key: K) -> Result<()>
    where
        K: AsRef<[u8]>,
    {
        self.db_mut().remove(key.as_ref());
        Ok(())
    }

    fn persist(&mut self) -> Result<()> {
        Ok(())
    }
}

