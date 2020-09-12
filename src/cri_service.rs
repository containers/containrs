use crate::storage::default_key_value_storage::DefaultKeyValueStorage;

#[derive(Clone)]
pub struct CRIService {
    storage: DefaultKeyValueStorage,
}

impl CRIService {
    pub fn new(storage: DefaultKeyValueStorage) -> Self {
        Self { storage }
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
