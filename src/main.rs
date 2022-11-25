#![warn(rust_2018_idioms)]
#![allow(dead_code)]

mod cli;
mod id;
mod kbucket;
mod node;
mod routing;
mod rpc;
mod storage;

#[tokio::main]
async fn main() {
    // Runtime::new().start().await.unwrap()
    // Move this into Node::new()?
    let mut n1 = node::Node::new(String::from("localhost:8080"))
        .await
        .unwrap();

    let mut n2 = node::Node::new(String::from("localhost:8081"))
        .await
        .unwrap();

    let (rh1, ph1, remh1) = n1.start();
    let (rh2, ph2, remh2) = n2.start();

    let res = n1.send(rpc::RequestPayload::Ping, &n2.node_info).await;
    // let res = n2.send(rpc::RequestPayload::Ping).await;
    println!("response: {:?}", res);
    tokio::join!(rh1, ph1, remh1, rh2, ph2, remh2);
}
