#![warn(rust_2018_idioms)]

mod bucket;
mod cli;
mod id;
mod lru;
mod node;
mod rpc;

use rpc::{Message, RequestHandle};

use crate::node::*;

// basic: start two nodes, have one ping the other, log and send back using mpsc
#[tokio::main]
async fn main() {
    let s1 = tokio::net::UdpSocket::bind("localhost:8080").await.unwrap();
    let s2 = tokio::net::UdpSocket::bind("localhost:8081").await.unwrap();

    let n1 = Node::new(s1).await;
    let n2 = Node::new(s2).await;

    let m1 = Message::Request(RequestHandle {
        source: n1.id.clone(),
        request: rpc::RequestPayload::Ping,
    });

    n1.rpc
        .send(
            &m1,
            &bucket::NodeInfo {
                address: "localhost:8081".to_string(),
                id: n1.id.clone(),
            },
        )
        .await;
}
