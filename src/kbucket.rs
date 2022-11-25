use std::collections::VecDeque;

use crate::{id::Id, routing::NodeInfo};

/// Maximum length of a `KBucket` before it is required to be split
pub const KBUCKET_MAX_LENGTH: usize = 20;

#[derive(Debug, Clone)]
pub struct KBucket(pub VecDeque<NodeInfo>);

impl KBucket {
    pub fn new() -> Self {
        KBucket(VecDeque::new())
    }

    /// Upsert a [`NodeInfo`] into the `KBucket`. Moving existing values to the tail
    pub fn upsert(&mut self, x: NodeInfo) {
        if self.0.contains(&x) {
            self.0.remove(
                self.0
                    .iter()
                    .position(|y| *y == x)
                    .expect("node info needle not found"),
            );
        }

        self.0.push_back(x);
        if self.0.len() > KBUCKET_MAX_LENGTH {
            self.0.pop_front();
        }
    }

    /// Check if the [`NodeInfo`] is contained within the `KBucket`
    pub fn contains(&self, x: &NodeInfo) -> bool {
        self.0.iter().any(|y| y == x)
    }

    /// Remove a [`NodeInfo`] from the `KBucket`
    pub fn remove(&mut self, x: &NodeInfo) -> Option<NodeInfo> {
        self.0
            .iter()
            .position(|y| y == x)
            .map(|y| self.0.remove(y))
            .flatten()
    }

    /// Split the `KBucket` at the given `distance`, returning a new `KBucket` which contain nodes further away than the distance
    pub fn split(&mut self, id: &Id, distance: usize) -> KBucket {
        let (old, new) = self
            .0
            .drain(..)
            .partition(|ni| ni.id.distance(id) == distance);
        self.0 = old;
        KBucket(new)
    }

    /// The number of nodes within the `KBucket`
    pub fn size(&self) -> usize {
        self.0.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let mut kb: KBucket = KBucket::new();
        let x = NodeInfo {
            id: Id::random(),
            address: "localhost:8080".to_string(),
        };
        let y = NodeInfo {
            id: Id::random(),
            address: "localhost:8081".to_string(),
        };
        kb.upsert(x.clone());
        assert_eq!(kb.0.len(), 1);
        kb.upsert(x.clone());
        assert_eq!(kb.0.len(), 1);
        kb.upsert(y.clone());
        assert_eq!(kb.0, vec![x.clone(), y.clone()]);
        kb.upsert(x.clone());
        assert_eq!(kb.0, vec![y.clone(), x.clone()]);
        kb.remove(&x);
        assert_eq!(kb.0, vec![y]);
    }
}
