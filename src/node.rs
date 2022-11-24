use std::{collections::HashMap, sync::Arc, time::Duration};

use tokio::{
    sync::{
        mpsc::{self, Receiver, Sender},
        Mutex,
    },
    task::JoinHandle,
    time,
};

use crate::{
    id::Id,
    routing::{NodeInfo, RoutingTable},
    rpc::{Message, RequestHandle, RequestPayload, ResponseHandle, ResponsePayload, Rpc},
    storage::Store,
};

pub struct Node {
    pub node_info: Arc<NodeInfo>,
    pub router: Arc<Mutex<RoutingTable>>,
    pub store: Arc<Mutex<Store<String, String>>>,
    /// Requests pending responses
    pub pending: Arc<Mutex<HashMap<Id, Sender<ResponseHandle>>>>,
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
        let router = Arc::clone(&self.router);
        let store = Arc::clone(&self.store);
        let rpc = Arc::clone(&self.rpc);
        let local_node_info = Arc::clone(&self.node_info);

        let process_handle = tokio::spawn(async move {
            loop {
                for message in rx.recv().await {}
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
                let mut store = store.lock().await;
                store.remove_stale();
                drop(store);
                interval.tick().await;
            }
        });

        remover_handle
    }
    // Send a request and get a response
    // fn request() -> response {

    // }

    /// Handle a request
    fn process_request(message: RequestHandle) {
        // match message.request {
        //     RequestPayload::Ping => {
        //         let destination = source;

        //         let response = Message::Response(ResponseHandle {
        //             receiver: local_node_info.as_ref().clone(),
        //             request: RequestPayload::Ping,
        //             response: ResponsePayload::Pong,
        //         });

        //         println!(
        //             "{:?} received ping from {:?}",
        //             local_node_info.id, source.id
        //         );

        //         rpc.send(&response, &destination).await;

        //         println!("{:?} sent pong to {:?}", local_node_info.id, source.id)
        //     }
        // }
    }

    /// Handle a reponse
    fn process_response(message: ResponseHandle) {
        // Lookup in the pending requests, notify requestee
        unimplemented!()
    }

    /// Send a request and wait and return a response.
    fn send(request: RequestPayload) -> Option<ResponseHandle> {
        unimplemented!()
    }

    // match message {
    //     Message::Request(RequestHandle {
    //         request,
    //         ref source,
    //     }) => match request {},
    //     Message::Response(ResponseHandle {
    //         receiver,
    //         request,
    //         response,
    //     }) => {
    //         println!(
    //             "{:?} received pong from {:?}",
    //             local_node_info.id, receiver.id
    //         );

    //         let request = Message::Request(RequestHandle {
    //             source: local_node_info.as_ref().clone(),
    //             request: RequestPayload::Ping,
    //         });

    //         rpc.send(&request, &receiver).await;
    //         println!("{:?} sent ping to {:?}", local_node_info.id, receiver.id)
    //     }
    // }
}
