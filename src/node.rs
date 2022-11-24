use std::{collections::HashMap, sync::Arc, time::Duration};

use tokio::{
    sync::{
        mpsc::{self, channel, Receiver, Sender},
        Mutex,
    },
    task::JoinHandle,
    time::{self},
};

use crate::{
    id::Id,
    routing::{NodeInfo, RoutingTable},
    rpc::{Message, RequestHandle, RequestPayload, ResponseHandle, ResponsePayload, Rpc},
    storage::Store,
};

#[derive(Clone)]
pub struct Node {
    pub node_info: NodeInfo,
    pub router: Arc<Mutex<RoutingTable>>,
    pub store: Arc<Mutex<Store<String, String>>>,
    /// Requests pending responses
    // XXX: Does this just need the payload or is the handle itself required?
    pub pending: Arc<Mutex<HashMap<Id, Sender<ResponsePayload>>>>,
    pub rpc: Arc<Rpc>,
}

impl Node {
    /// Create a new node with a random [`Id`] and an empty [`RoutingTable`], start the [`Rpc`]
    pub async fn new(address: String) -> Self {
        let id = Id::random();

        let node_info = NodeInfo {
            id: id,
            address: address.clone(),
        };
        let socket = tokio::net::UdpSocket::bind(&address).await.unwrap();
        let store = Arc::new(Mutex::new(Store::new()));
        let router = Arc::new(Mutex::new(RoutingTable::new(node_info.clone())));
        let rpc = Arc::new(Rpc::new(Arc::new(socket)));
        let pending = Arc::new(Mutex::new(HashMap::new()));

        Self {
            node_info,
            rpc,
            router,
            store,
            pending,
        }
    }

    /// Start receive and process services
    pub fn start(&self) -> (JoinHandle<()>, JoinHandle<()>, JoinHandle<()>) {
        let (tx, rx) = mpsc::channel(50);
        let receive_handle = self.rpc.receive(tx);
        let process_handle = self.process(rx);
        let remover_handle = self.remover();
        (receive_handle, process_handle, remover_handle)
    }

    /// Process incoming messages
    fn process(&self, mut rx: Receiver<Message>) -> JoinHandle<()> {
        // This might panic as its a mutable reference while main thread is doing shit
        let mut node = self.clone();
        let process_handle = tokio::spawn(async move {
            loop {
                for message in rx.recv().await {
                    match message {
                        Message::Request(request_handle) => {
                            node.process_request(request_handle).await
                        }
                        Message::Response(response_handle) => {
                            node.process_response(response_handle).await
                        }
                    }
                }
            }
        });

        process_handle
    }

    /// Start the service to remove stale indexes every hour
    pub fn remover(&self) -> JoinHandle<()> {
        let store = Arc::clone(&self.store);
        let remover_handle = tokio::spawn(async move {
            let mut interval = time::interval(Duration::from_secs(60 * 60));
            loop {
                {
                    let mut store = store.lock().await;
                    store.remove_stale();
                }
                interval.tick().await;
            }
        });

        remover_handle
    }

    /// Handle a request, send a response
    async fn process_request(&mut self, message: RequestHandle) {
        println!("processing request");
        match message.request {
            RequestPayload::Ping => {
                {
                    let mut router = self.router.lock().await;
                    router.upsert(message.source.clone());
                }
                let response = Message::Response(ResponseHandle {
                    id: Id::random(),
                    source: message.source.clone(),
                    request_id: message.id,
                    response: ResponsePayload::Pong,
                });

                self.rpc.send(&response, &message.source).await;
            }
        }
    }

    /// Handle a reponse
    // Lookup in the pending requests, notify requestee
    async fn process_response(&mut self, message: ResponseHandle) {
        let mut pending = self.pending.lock().await;
        {
            let mut router = self.router.lock().await;
            router.upsert(message.source);
        }

        match pending.remove(&message.request_id) {
            Some(tx) => {
                println!("sending response back to send fn {:?}", message.id);
                tx.send(message.response).await.unwrap();
            }
            None => {
                eprintln!("Received response for request that has not been tracked")
            }
        }
    }

    /// Send a request and wait and return a response.
    pub async fn send(&mut self, request: RequestPayload) -> Option<ResponsePayload> {
        let request_id = Id::random();
        let message = Message::Request(RequestHandle {
            id: request_id.clone(),
            source: self.node_info.to_owned(),
            request: request,
        });
        // TODO: Change this to a oneshot channel
        let (tx, mut rx) = channel(50);
        {
            let mut pending = self.pending.lock().await;
            pending.insert(request_id, tx);
        }
        self.rpc.send(&message, &self.node_info.clone()).await;

        let val = rx.recv().await;
        val
    }
}
