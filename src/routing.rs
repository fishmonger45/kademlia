use std::{cmp, collections::VecDeque, net::SocketAddr, ops::Index, sync::Arc};

use serde::{Deserialize, Serialize};

use crate::{
    id::Id,
    kbucket::{KBucket, KBUCKET_MAX_LENGTH},
};

// Maximum number of KBuckets in the routing table
pub const ROUTING_TABLE_MAX_LENGTH: usize = 15;

#[derive(PartialEq, Eq, Deserialize, Serialize, Debug, Clone)]
pub struct NodeInfo {
    pub id: Id,
    /// In the form `ip:port`
    pub address: String,
}

#[derive(Debug)]
pub struct RoutingTable {
    kbuckets: Vec<KBucket>,
    node_info: NodeInfo,
}

impl RoutingTable {
    /// Create a new `RoutingTable` with a single empty [`KBucket`]
    pub fn new(node_info: NodeInfo) -> Self {
        Self {
            kbuckets: vec![KBucket::new()],
            node_info: node_info,
        }
    }

    /// Upsert a node into the `RoutingTable`, splitting [`KBucket`] as nessesary
    pub fn upsert(&mut self, node_info: NodeInfo) -> bool {
        let mut index = cmp::min(
            self.node_info.id.distance(&node_info.id),
            self.kbuckets.len() - 1,
        );

        if self.kbuckets[index].contains(&node_info) {
            self.kbuckets[index].upsert(node_info);
            return true;
        } else {
            loop {
                if self.kbuckets[index].size() < KBUCKET_MAX_LENGTH {
                    self.kbuckets[index].upsert(node_info.clone());
                    return true;
                }

                let is_last_bucket = index == self.kbuckets.len() - 1;
                let is_full = self.kbuckets.len() == ROUTING_TABLE_MAX_LENGTH;

                // Only last bucket can be split and bucket must be full
                if !is_last_bucket || is_full {
                    return false;
                }

                let new = self.kbuckets[index].split(&self.node_info.id, index);
                self.kbuckets.push(new);

                index = cmp::min(
                    self.node_info.id.distance(&node_info.id),
                    self.kbuckets.len() - 1,
                );
            }
        }
    }

    /// Find a [`NodeInfo`] in the `RoutingTable` from an `Id`
    pub fn find(&self, id: &Id) -> Option<NodeInfo> {
        for kb in self.kbuckets.iter() {
            match kb.find(id) {
                None => continue,
                x => return x,
            }
        }

        None
    }

    /// Get the `n` closest nodes to [`Id`]
    pub fn closest(&self, id: &Id, n: usize) -> Vec<NodeInfo> {
        let mut index = cmp::min(
            self.node_info.id.distance(&self.node_info.id),
            self.kbuckets.len() - 1,
        );

        let mut closest = Vec::new();
        closest.extend_from_slice(self.kbuckets[index].0.clone().make_contiguous());

        while index < self.kbuckets.len() && closest.len() < n {
            for i in (index + 1)..self.kbuckets.len() {
                let required = std::cmp::min(self.kbuckets[i].size(), n - closest.len());
                closest
                    .extend_from_slice(&self.kbuckets[i].0.clone().make_contiguous()[..required]);
            }
            index += 1
        }

        closest.sort_by_key(|node_info| node_info.id.distance(id));

        closest
    }

    /// Number of [`KBuckets`] in the `RoutingTable`
    pub fn size(&self) -> usize {
        self.kbuckets.len()
    }

    /// Remove a node from the `RoutingTable` and return it
    pub fn remove(&mut self, node_info: &NodeInfo) -> Option<NodeInfo> {
        let idx = std::cmp::min(
            self.node_info.id.distance(&node_info.id),
            self.kbuckets.len() - 1,
        );

        self.kbuckets[idx].remove(node_info)
    }
}

impl Index<Id> for RoutingTable {
    type Output = KBucket;
    fn index(&self, id: Id) -> &Self::Output {
        let idx = cmp::min(self.node_info.id.distance(&id), self.kbuckets.len() - 1);

        return &self.kbuckets[idx];
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn index() {
        let mut rt = RoutingTable::new(NodeInfo {
            id: Id::new([1u8; 20]),
            address: "localhost:8080".to_string(),
        });

        let n1 = NodeInfo {
            id: Id::new([3u8; 20]),
            address: "localhost:8081".to_string(),
        };
        rt.upsert(n1);
        println!("{:?}", rt)
    }

    #[test]
    fn find() {
        let mut rt = RoutingTable::new(NodeInfo {
            id: Id::random(),
            address: "localhost:8080".to_string(),
        });

        let id = Id::random();
        let n1 = NodeInfo {
            id: id.clone(),
            address: "localhost:8081".to_string(),
        };
        rt.upsert(n1.clone());
        assert_eq!(rt.find(&id), Some(n1))
    }
}
