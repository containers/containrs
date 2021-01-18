//! The default key value storage implementation for storing arbitrary data.

use crate::storage::KeyValueStorage;
use anyhow::{Context, Result};
use getset::Getters;
use log::trace;
use serde::{de::DeserializeOwned, Serialize};
use sled::Db;
use std::{convert::AsRef, path::Path};

#[derive(Debug, Clone, Getters)]
/// A default key value storage implementation
pub struct DefaultKeyValueStorage {
    #[get]
    /// The internal database.
    db: Db,
}

impl KeyValueStorage for DefaultKeyValueStorage {
    /// Open the database, whereas the `Path` has to be a directory.
    fn open(path: &Path) -> Result<Self> {
        trace!("Opening storage {}", path.display());
        Ok(Self {
            db: sled::open(path)
                .with_context(|| format!("failed to open storage path {}", path.display()))?,
        })
    }

    fn get<K, V>(&self, key: K) -> Result<Option<V>>
    where
        K: AsRef<[u8]>,
        V: DeserializeOwned,
    {
        match self
            .db()
            .get(key)
            .context("failed to retrieve value for key")?
        {
            None => Ok(None),
            Some(value) => {
                trace!("Got result from storage (len = {})", value.len());
                Ok(Some(
                    rmp_serde::from_slice(&value).context("deserialize value")?,
                ))
            }
        }
    }

    fn insert<K, V>(&mut self, key: K, value: V) -> Result<()>
    where
        K: AsRef<[u8]>,
        V: Serialize,
    {
        self.db()
            .insert(
                key,
                rmp_serde::to_vec(&value).context("failed to serialize value")?,
            )
            .context("failed to insert key and value")?;
        trace!("Inserted item into storage (count = {})", self.db().len());
        Ok(())
    }

    fn remove<K>(&mut self, key: K) -> Result<()>
    where
        K: AsRef<[u8]>,
    {
        self.db().remove(key)?.context("failed to remove value")?;
        trace!("Removed item from storage (count = {})", self.db().len());
        Ok(())
    }

    fn persist(&mut self) -> Result<()> {
        self.db().flush().context("failed to persist db")?;
        trace!("Persisted storage");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;
    use tempfile::TempDir;

    #[test]
    fn get_existing_value() -> Result<()> {
        let dir = TempDir::new()?;
        let mut db = DefaultKeyValueStorage::open(dir.path())?;

        let (k, v) = ("key", "value");
        db.insert(k, v)?;
        let res: String = db.get(k)?.context("value is none")?;
        assert_eq!(res, v);
        Ok(())
    }

    #[test]
    fn get_nonexisting_value() -> Result<()> {
        let dir = TempDir::new()?;
        let db = DefaultKeyValueStorage::open(dir.path())?;

        assert!(db.get::<_, String>("key")?.is_none());
        Ok(())
    }

    #[test]
    fn remove_value() -> Result<()> {
        let dir = TempDir::new()?;
        let mut db = DefaultKeyValueStorage::open(dir.path())?;

        let (k, v) = ("key", "value");
        db.insert(k, v)?;
        db.remove(k)?;
        assert!(db.get::<_, String>(k)?.is_none());

        Ok(())
    }

    #[test]
    fn persist() -> Result<()> {
        let dir = TempDir::new()?;
        let mut db = DefaultKeyValueStorage::open(dir.path())?;

        db.insert("key", "value")?;
        db.persist()
    }

    #[test]
    fn insert_values() -> Result<()> {
        let dir = TempDir::new()?;
        let mut db = DefaultKeyValueStorage::open(dir.path())?;

        #[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
        struct NewValue(String);

        let k1 = vec![1, 2, 3];
        let v1 = NewValue("value".into());

        let v2 = "value 2";
        let k2 = vec![3, 2, 1];

        db.insert(k1.clone(), v1.clone())?;
        assert_eq!(
            db.get::<_, NewValue>(k1)?.context("value for k1 is none")?,
            v1
        );

        db.insert(k2.clone(), v2.clone())?;
        assert_eq!(
            db.get::<_, String>(k2)?.context("value for k2 is none")?,
            v2
        );
        Ok(())
    }

    #[test]
    fn open_twice() -> Result<()> {
        let dir = TempDir::new()?;

        let mut db1 = DefaultKeyValueStorage::open(dir.path())?;
        let db2 = db1.clone();

        let (k, v) = ("key", "value");

        db1.insert(k, v)?;

        let res1: String = db1.get(k)?.context("value 1 is none")?;
        assert_eq!(res1, v);

        let res2: String = db2.get(k)?.context("value 2 is none")?;
        assert_eq!(res2, v);

        Ok(())
    }
}
