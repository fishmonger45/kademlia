use std::sync::Arc;

use crate::{id::Id, routing::NodeInfo};
use serde::{Deserialize, Serialize};
use tokio::{net::UdpSocket, sync::mpsc::Sender, task::JoinHandle};

/// Maximum message size sent over the wire
pub const MESSAGE_SIZE: usize = 2000;

/// Request message payload
#[derive(Serialize, Deserialize, Debug)]
pub enum RequestPayload {
    Ping,
    Store { key: String, value: String },
    FindNode { id: Id },
}

/// Response message payload
#[derive(Serialize, Deserialize, Debug)]
pub enum ResponsePayload {
    Pong,
    FindNode { closest: Vec<NodeInfo> },
}

/// Wraps request with sender details
#[derive(Serialize, Deserialize, Debug)]
pub struct RequestHandle {
    pub id: Id,
    pub source: NodeInfo,
    pub request: RequestPayload,
}

/// Wraps responses with the response to a request and reciever details
#[derive(Serialize, Deserialize, Debug)]
pub struct ResponseHandle {
    pub id: Id,
    pub source: NodeInfo,
    pub request_id: Id,
    pub response: ResponsePayload,
}

/// A message that is sent between nodes
#[derive(Serialize, Deserialize, Debug)]
pub enum Message {
    Request(RequestHandle),
    Response(ResponseHandle),
}

// Protocol handler for sending and recieving messages
pub struct Rpc {
    socket: Arc<UdpSocket>,
}

impl Rpc {
    // Create a new `Rpc` handler
    pub fn new(socket: Arc<UdpSocket>) -> Self {
        Self { socket }
    }
    /// Listen for messages and send them to process
    pub fn receive(&self, tx: Sender<Message>) -> JoinHandle<()> {
        let socket = Arc::clone(&self.socket);
        let receive_handle = tokio::spawn(async move {
            let mut buffer = [0u8; MESSAGE_SIZE];
            loop {
                let (x, _) = socket
                    .recv_from(&mut buffer)
                    .await
                    .expect("failed to receive message");

                let message = serde_json::from_slice::<Message>(&buffer[..x])
                    .expect("failed to deserialize message");

                tx.send(message)
                    .await
                    .expect("failed to send received message to handler");
            }
        });

        receive_handle
    }

    /// Send a message to a node
    pub async fn send(&self, message: &Message, node_info: &NodeInfo) {
        println!("sending message");
        let buffer = serde_json::to_vec(message).expect("failed to serialize message");
        self.socket
            .send_to(&buffer, &node_info.address)
            .await
            .expect("failed to send message");
    }
}
