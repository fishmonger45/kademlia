use serde::{Deserialize, Serialize};
use tokio::net::unix::SocketAddr;

use crate::{id::Id, lru::Lru};

#[derive(PartialEq, Eq, Deserialize, Serialize)]
pub struct NodeInfo {
    // We can't use tokio::net::SocketAddr because thats not Eq, we can use strings instead
    pub address: String,
    pub id: Id,
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
