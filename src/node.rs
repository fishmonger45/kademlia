use std::{collections::HashMap, sync::Arc};

use tokio::{
    net::UdpSocket,
    sync::{
        mpsc::{self, Receiver},
        Mutex,
    },
    task::JoinHandle,
};

use crate::{
    id::Id,
    routing::{NodeInfo, RoutingTable},
    rpc::{Message, RequestHandle, RequestPayload, ResponseHandle, ResponsePayload, Rpc},
};

// TODO: Expiring 24hrs store
pub type Store = HashMap<String, String>;

pub struct Node {
    pub node_info: Arc<NodeInfo>,
    pub router: Arc<Mutex<RoutingTable>>,
    pub store: Arc<Mutex<Store>>,
    pub rpc: Arc<Rpc>,
}

impl Node {
    /// Create a new node with a random [`Id`] and an empty [`RoutingTable`], start the [`Rpc`]
    pub async fn new(address: String) -> Self {
        let id = Id::random();

        let node_info = Arc::new(NodeInfo {
            id: id,
            address: address.clone(),
        });
        let socket = tokio::net::UdpSocket::bind(&address).await.unwrap();
        let store = Arc::new(Mutex::new(Store::new()));
        let router = Arc::new(Mutex::new(RoutingTable::new(Arc::clone(&node_info))));
        let rpc = Arc::new(Rpc::new(Arc::new(socket)));

        Self {
            node_info,
            rpc,
            router,
            store,
        }
    }

    /// Start receive and process services
    pub fn start(&self) -> (JoinHandle<()>, JoinHandle<()>) {
        let (tx, rx) = mpsc::channel(50);
        let receive_handle = self.rpc.receive(tx);
        let process_handle = self.process(rx);
        (receive_handle, process_handle)
    }

    /// Process incoming messages
    fn process(&self, mut rx: Receiver<Message>) -> JoinHandle<()> {
        let router = Arc::clone(&self.router);
        let store = Arc::clone(&self.store);
        let rpc = Arc::clone(&self.rpc);
        let local_node_info = Arc::clone(&self.node_info);

        let process_handle = tokio::spawn(async move {
            loop {
                for message in rx.recv().await {
                    match message {
                        Message::Request(RequestHandle {
                            request,
                            ref source,
                        }) => match request {
                            RequestPayload::Ping => {
                                let destination = source;

                                let response = Message::Response(ResponseHandle {
                                    receiver: local_node_info.as_ref().clone(),
                                    request: RequestPayload::Ping,
                                    response: ResponsePayload::Pong,
                                });

                                println!(
                                    "{:?} received ping from {:?}",
                                    local_node_info.id, source.id
                                );

                                rpc.send(&response, &destination).await;

                                println!("{:?} sent pong to {:?}", local_node_info.id, source.id)
                            }
                        },
                        Message::Response(ResponseHandle {
                            receiver,
                            request,
                            response,
                        }) => {
                            println!(
                                "{:?} received pong from {:?}",
                                local_node_info.id, receiver.id
                            );

                            let request = Message::Request(RequestHandle {
                                source: local_node_info.as_ref().clone(),
                                request: RequestPayload::Ping,
                            });

                            rpc.send(&request, &receiver).await;
                            println!("{:?} sent ping to {:?}", local_node_info.id, receiver.id)
                        }
                    }
                }
            }
        });

        process_handle
    }
}
