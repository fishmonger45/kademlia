#![warn(rust_2018_idioms)]

mod bucket;
mod cli;
mod id;
mod lru;
mod node;
mod rpc;

use std::net::{SocketAddr, ToSocketAddrs};

use bucket::NodeInfo;
use node::Node;
use rpc::{Message, RequestHandle};

// basic: start two nodes, have one ping the other, log and send back using mpsc
#[tokio::main]
async fn main() {
    // Move this into Node::new()?
    let n1 = Node::new(String::from("localhost:8080")).await;
    let n2 = Node::new(String::from("localhost:8081")).await;

    let (rh1, ph1) = n1.start();
    let (rh2, ph2) = n2.start();

    let m1 = Message::Request(RequestHandle {
        source: n1.node_info.as_ref().clone(),
        request: rpc::RequestPayload::Ping,
    });

    n1.rpc.send(&m1, &n2.node_info).await;
    rh2.await;
    ph2.await;
}
