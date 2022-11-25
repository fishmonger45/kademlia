use std::{collections::HashMap, error::Error, fmt::Display, sync::Arc, time::Duration};

use tokio::{
    sync::{
        mpsc::{self},
        oneshot::{self, Sender},
        Mutex,
    },
    task::JoinHandle,
    time::{self, timeout},
};

use crate::{
    id::Id,
    routing::{NodeInfo, RoutingTable},
    rpc::{Message, RequestHandle, RequestPayload, ResponseHandle, ResponsePayload, Rpc},
    storage::Store,
};

/// Time to wait for a response before timing out
const RESPONSE_TIMEOUT: Duration = Duration::new(1, 0);

#[derive(Clone)]
pub struct Node {
    pub node_info: NodeInfo,
    pub router: Arc<Mutex<RoutingTable>>,
    pub store: Arc<Mutex<Store<String, String>>>,
    pub pending: Arc<Mutex<HashMap<Id, oneshot::Sender<ResponsePayload>>>>,
    pub rpc: Arc<Rpc>,
}

impl Node {
    /// Create a new node with a random [`Id`] and an empty [`RoutingTable`], start the [`Rpc`]
    pub async fn new(address: String) -> Result<Self, Box<dyn Error>> {
        let id = Id::random();

        let node_info = NodeInfo {
            id: id,
            address: address.clone(),
        };
        let socket = tokio::net::UdpSocket::bind(&address).await?;
        let store = Arc::new(Mutex::new(Store::new()));
        let router = Arc::new(Mutex::new(RoutingTable::new(node_info.clone())));
        let rpc = Arc::new(Rpc::new(Arc::new(socket)));
        let pending = Arc::new(Mutex::new(HashMap::new()));

        Ok(Self {
            node_info,
            rpc,
            router,
            store,
            pending,
        })
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
    fn process(&self, mut rx: mpsc::Receiver<Message>) -> JoinHandle<()> {
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

    /// Handle a request and send a response
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
            RequestPayload::Store { key, value } => {
                {
                    let mut router = self.router.lock().await;
                    router.upsert(message.source.clone());
                    let mut store = self.store.lock().await;
                    store.upsert(key, value);
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

    /// Handle a request reponse, looking up pending requests, notify requestee
    async fn process_response(&mut self, message: ResponseHandle) {
        let mut pending = self.pending.lock().await;
        {
            let mut router = self.router.lock().await;
            router.upsert(message.source);
        }

        match pending.remove(&message.request_id) {
            Some(tx) => {
                println!("sending response back to send fn {:?}", message.id);
                tx.send(message.response)
                    .expect("failed to send response via oneshot");
            }
            None => {
                eprintln!("Received response for request that has not been tracked")
            }
        }
    }

    /// Send a request and wait and return a response.
    pub async fn send(
        &mut self,
        request: RequestPayload,
        destination: &NodeInfo,
    ) -> Option<ResponsePayload> {
        let request_id = Id::random();
        let message = Message::Request(RequestHandle {
            id: request_id.clone(),
            source: self.node_info.to_owned(),
            request: request,
        });
        // TODO: Change this to a oneshot channel
        let (tx, rx) = oneshot::channel();
        {
            let mut pending = self.pending.lock().await;
            pending.insert(request_id, tx);
        }
        self.rpc.send(&message, destination).await;
        let response = timeout(RESPONSE_TIMEOUT, rx)
            .await
            .expect("waiting for request response timed out");
        Some(response.expect("failed to receive response via oneshot"))
    }
}

impl Display for Node {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.node_info.id.hex())
    }
}
