use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use tokio::net::UdpSocket;

use crate::{bucket::RoutingTable, id::Id, rpc::Rpc};

// TODO: Expiring 24hrs store
type Store = HashMap<String, String>;

pub struct Node {
    pub id: Id,
    pub router: RoutingTable,
    pub store: Arc<Mutex<Store>>,
    pub rpc: Rpc,
}

impl Node {
    /// Create a new node with a random [`Id`] and an empty [`RoutingTable`], start the [`Rpc`]
    pub async fn new(socket: UdpSocket) -> Self {
        let rpc = Rpc::new(socket);
        rpc.receive().await;
        Self {
            id: Id::random(),
            router: RoutingTable::new(),
            store: Arc::new(Mutex::new(Store::new())),
            rpc,
        }
    }
}
