use std::{
    collections::{BTreeMap, HashMap, HashSet},
    hash::Hash,
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
    K: Hash + PartialEq + Eq + Ord + Copy,
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
        self.store.insert(k, v);
        self.times.insert(k, Instant::now());
    }

    /// Remove all stale entries from the store
    pub fn remove_stale(&mut self) {
        let now = Instant::now();
        let mut keys = Vec::new();

        for (k, t) in self.times.iter() {
            if now.duration_since(*t) >= STALE_DURATION {
                self.store.remove(k).expect("");
                keys.push(k);
            }
        }

        for k in keys {
            self.store.remove(k);
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
    }
}
