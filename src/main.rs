#![warn(rust_2018_idioms)]

mod cli;
mod id;
mod kbucket;
mod node;
mod routing;
mod rpc;
mod storage;
use node::Node;

// basic: start two nodes, have one ping the other, log and send back using mpsc
#[tokio::main]
async fn main() {
    // Move this into Node::new()?
    let mut n1 = Node::new(String::from("localhost:8080")).await;
    let mut n2 = Node::new(String::from("localhost:8081")).await;

    let (rh1, ph1, remh1) = n1.start();
    let (rh2, ph2, remh2) = n2.start();

    let res = n1.send(rpc::RequestPayload::Ping).await;
    println!("response: {:?}", res);
    tokio::join!(rh1, ph1, remh1, rh2, ph2, remh2);
}
