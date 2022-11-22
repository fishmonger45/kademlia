use std::net::SocketAddr;

use serde::{Deserialize, Serialize};

use crate::{id::Id, lru::Lru};

#[derive(PartialEq, Eq, Deserialize, Serialize, Debug, Clone)]
pub struct NodeInfo {
    pub id: Id,
    /// In the form `ip:port`
    pub address: String,
}

pub type KBucket = Lru<NodeInfo>;

pub struct RoutingTable {
    kbuckets: Vec<KBucket>,
}

impl RoutingTable {
    pub fn new() -> Self {
        Self {
            kbuckets: Vec::new(),
        }
    }
}

impl RoutingTable {}
