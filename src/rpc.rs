use std::sync::Arc;

use crate::{bucket::NodeInfo, id::Id};
use serde::{Deserialize, Serialize};
use tokio::{net::UdpSocket, sync::mpsc::Sender};

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
    pub source: Id,
    pub request: RequestPayload,
}

/// Wraps responses with the response to a request and reciever details
#[derive(Serialize, Deserialize, Debug)]
pub struct ResponseHandle {
    destination: Id,
    request: RequestPayload,
    response: ResponsePayload,
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
    // TODO: For now we are not handling messages that are received
    // async fn new(socket: UdpSocket, tx: Sender<Message>) {

    // Create a new [`Rpc`] handler
    pub fn new(socket: UdpSocket) -> Self {
        Self {
            socket: Arc::new(socket),
        }
    }
    /// Listen for messages and send them to a request handler
    pub async fn receive(&self) {
        let socket = Arc::clone(&self.socket);
        println!("spawning receive future");
        tokio::spawn(async move {
            let mut buffer = [0u8; 2 << 12];
            loop {
                let (x, _) = socket
                    .recv_from(&mut buffer)
                    .await
                    .expect("failed to recieve message");

                println!("received {x} bytes");

                let message = serde_json::from_slice::<Message>(&buffer)
                    .expect("failed to deserialize message");

                println!("recieved {:?}", message);
            }
        });
    }

    /// Send a message to another node
    pub async fn send(&self, message: &Message, node_info: &NodeInfo) {
        let buffer = serde_json::to_vec(message).expect("failed to serialize message");
        println!("sending message of size {:?}", buffer.len());

        self.socket
            .send_to(&buffer, &node_info.address)
            .await
            .expect("failed to send message");

        println!("sent {:?}", message);
    }
}
