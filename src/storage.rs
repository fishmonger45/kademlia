use std::{
    collections::{BTreeMap, HashMap},
    hash::Hash,
    sync::Arc,
    time::{Duration, Instant},
};

//// 24 hour duration before a key is removed
pub const STALE_DURATION: Duration = Duration::new(24 * 60 * 60, 0);

pub struct Store<K, V> {
    store: HashMap<K, V>,
    times: BTreeMap<K, Instant>,
}

impl<K, V> Store<K, V>
where
    K: Hash + PartialEq + Eq + Ord + Clone + Send + Sync,
    V: Clone + Send + Sync,
{
    /// Create an empty store
    pub fn new() -> Self {
        Self {
            store: HashMap::new(),
            times: BTreeMap::new(),
        }
    }

    /// Upsert a value to the store
    pub fn upsert(&mut self, k: K, v: V) {
        self.store.insert(k.clone(), v);
        self.times.insert(k, Instant::now());
    }

    pub fn get(&self, k: &K) -> Option<V> {
        self.store.get(&k).map(|v| v.clone())
    }

    /// Remove all stale entries from the store
    pub fn remove_stale(&mut self) {
        let now = Instant::now();
        let mut keys = Vec::new();

        for (k, t) in self.times.iter() {
            if now.duration_since(*t) >= STALE_DURATION {
                keys.push(k.clone());
            }
        }

        for k in keys {
            self.times
                .remove(&k)
                .expect("failed to remove stale key from times");
            self.store
                .remove(&k)
                .expect("failed to remove stale key from store");
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn upsert() {
        let mut store = Store::<usize, usize>::new();
        store.upsert(0, 0);
        assert_eq!(store.get(&0), Some(0));
        // store.upsert(0, 0);
    }
}
