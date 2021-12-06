use crate::KeyValueStorage;
use anyhow::Result;
use getset::{Getters, MutGetters};
use std::collections::HashMap;

#[derive(Debug, Default, Clone, Getters, MutGetters)]
pub struct MemoryKeyValueStorage {
    #[getset(get, get_mut)]
    db: HashMap<Vec<u8>, Vec<u8>>,
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

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Context;
    use serde::{Deserialize, Serialize};

    #[test]
    fn get_existing_value() -> Result<()> {
        let mut db = MemoryKeyValueStorage::default();

        let (k, v) = ("key", "value");
        db.insert(k, v)?;
        let res: String = db.get(k)?.context("value is none")?;
        assert_eq!(res, v);
        Ok(())
    }

    #[test]
    fn get_nonexisting_value() -> Result<()> {
        let db = MemoryKeyValueStorage::default();

        assert!(db.get::<_, String>("key")?.is_none());
        Ok(())
    }

    #[test]
    fn remove_value() -> Result<()> {
        let mut db = MemoryKeyValueStorage::default();

        let (k, v) = ("key", "value");
        db.insert(k, v)?;
        db.remove(k)?;
        assert!(db.get::<_, String>(k)?.is_none());

        Ok(())
    }

    #[test]
    fn persist() -> Result<()> {
        let mut db = MemoryKeyValueStorage::default();

        db.insert("key", "value")?;
        db.persist()
    }

    #[test]
    fn insert_values() -> Result<()> {
        let mut db = MemoryKeyValueStorage::default();

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
}
