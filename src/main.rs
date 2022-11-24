#![warn(rust_2018_idioms)]

mod cli;
mod id;
mod kbucket;
mod node;
mod routing;
mod rpc;
mod storage;

use id::Id;
use node::Node;
use rpc::{Message, RequestHandle};

// basic: start two nodes, have one ping the other, log and send back using mpsc
#[tokio::main]
async fn main() {
    // Move this into Node::new()?
    let n1 = Node::new(String::from("localhost:8080")).await;
    let n2 = Node::new(String::from("localhost:8081")).await;

    let (rh1, ph1, remh1) = n1.start();
    let (rh2, ph2, remh2) = n2.start();

    let m1 = Message::Request(RequestHandle {
        id: Id::random(),
        source: n1.node_info.as_ref().clone(),
        request: rpc::RequestPayload::Ping,
    });

    n1.rpc.send(&m1, &n2.node_info).await;
    rh1.await;
    ph1.await;
    remh1.await;
    rh2.await;
    ph2.await;
    remh2.await;
}
