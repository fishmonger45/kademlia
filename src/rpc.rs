use std::sync::Arc;

use crate::{
    bucket::{NodeInfo, RoutingTable},
    id::Id,
    node::{Node, Store},
};
use serde::{Deserialize, Serialize};
use tokio::{
    net::UdpSocket,
    sync::{
        mpsc::{self, Receiver, Sender},
        Mutex,
    },
    task::JoinHandle,
};

/// Request message types
#[derive(Serialize, Deserialize, Debug)]
pub enum RequestPayload {
    Ping,
}

/// Response message types
#[derive(Serialize, Deserialize, Debug)]
pub enum ResponsePayload {
    Pong,
}

/// Wraps request with sender details
#[derive(Serialize, Deserialize, Debug)]
pub struct RequestHandle {
    pub source: NodeInfo,
    pub request: RequestPayload,
}

/// Wraps responses with the response to a request and reciever details
#[derive(Serialize, Deserialize, Debug)]
pub struct ResponseHandle {
    pub receiver: NodeInfo,
    pub request: RequestPayload,
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
    // Create a new [`Rpc`] handler
    pub fn new(socket: Arc<UdpSocket>) -> Self {
        Self { socket }
    }
    /// Listen for messages and send them to a handler
    pub fn receive(&self, tx: Sender<Message>) -> JoinHandle<()> {
        let socket = Arc::clone(&self.socket);
        let receive_handle = tokio::spawn(async move {
            let mut buffer = [0u8; 508];
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
        let buffer = serde_json::to_vec(message).expect("failed to serialize message");
        self.socket
            .send_to(&buffer, &node_info.address)
            .await
            .expect("failed to send message");
    }
}
