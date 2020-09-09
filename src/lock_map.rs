use std::{collections::HashMap, hash::Hash, sync::Arc};

use tokio::sync::{Mutex, OwnedMutexGuard, TryLockError};

#[derive(Default)]
pub struct LockMap<K: Hash + Eq> {
    inner: std::sync::Mutex<HashMap<K, Arc<Mutex<()>>>>,
}

pub struct LockMapGuard<'m, K: Hash + Eq> {
    map: &'m LockMap<K>,
    key: K,
    _guard: OwnedMutexGuard<()>,
}
impl<K: Hash + Eq> Drop for LockMapGuard<'_, K> {
    fn drop(&mut self) {
        self.map
            .inner
            .lock()
            .expect("lock map guard")
            .remove(&self.key);
    }
}

impl<K: Hash + Eq + Clone> LockMap<K> {
    fn mutex_by_key(&self, key: K) -> Arc<Mutex<()>> {
        let mut map = self.inner.lock().expect("lock map guard");
        let mutex = map
            .entry(key)
            .or_insert_with(|| Arc::new(Mutex::new(())))
            .clone();
        mutex
    }
    pub async fn lock(&self, key: K) -> LockMapGuard<'_, K> {
        let guard = self.mutex_by_key(key.clone()).lock_owned().await;
        LockMapGuard {
            map: self,
            key,
            _guard: guard,
        }
    }
    pub fn try_lock(&self, key: K) -> Result<LockMapGuard<'_, K>, TryLockError> {
        let guard = self.mutex_by_key(key.clone()).try_lock_owned()?;
        Ok(LockMapGuard {
            map: self,
            key,
            _guard: guard,
        })
    }
}
