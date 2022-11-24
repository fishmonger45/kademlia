use std::{collections::VecDeque, mem};

use crate::{id::Id, routing::NodeInfo};

pub const KBUCKET_MAX_LENGTH: usize = 20;

#[derive(Debug, Clone)]
pub struct KBucket(pub VecDeque<NodeInfo>);

impl KBucket {
    pub fn new() -> Self {
        KBucket(VecDeque::new())
    }

    /// Upsert a value in a [`KBucket`]. Moving existing values to the tail
    pub fn upsert(&mut self, x: NodeInfo) {
        if self.0.contains(&x) {
            self.0.remove(
                self.0
                    .iter()
                    .position(|y| *y == x)
                    .expect("needle not found"),
            );
        }
        self.0.push_back(x);
        if self.0.len() > KBUCKET_MAX_LENGTH {
            self.0.pop_front();
        }
    }

    /// Check if the node is contained within the [`KBucket`]
    pub fn contains(&self, x: &NodeInfo) -> bool {
        self.0.iter().any(|y| y == x)
    }

    /// Remove a given element from the [`KBucket`]
    pub fn remove(&mut self, x: &NodeInfo) -> Option<NodeInfo> {
        self.0
            .iter()
            .position(|y| y == x)
            .map(|y| self.0.remove(y))
            .flatten()
    }

    pub fn split(&mut self, id: &Id, idx: usize) -> KBucket {
        let (old, new) = self.0.drain(..).partition(|ni| ni.id.distance(id) == idx);
        self.0 = old;
        KBucket(new)
    }

    /// The number of nodes within the [`KBucket`]
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
