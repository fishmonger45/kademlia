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

    let res1 = n1.send(rpc::RequestPayload::Ping, &n2.node_info).await;
    let res2 = n2.send(rpc::RequestPayload::Ping, &n1.node_info).await;
    // let res = n2.send(rpc::RequestPayload::Ping).await;
    println!("res1: {:?}", res1);
    println!("res2: {:?}", res2);

    let res3 = n1
        .send(
            rpc::RequestPayload::Store {
                key: "hello".to_string(),
                value: "world".to_string(),
            },
            &n2.node_info,
        )
        .await;

    println!("res3: {:?}", res3);

    let res4 = n2.store.lock().await.get(&"hello".to_string());
    println!("res4: {:?}", res4);

    let res5 = n1
        .send(
            rpc::RequestPayload::FindNode {
                id: n1.node_info.id.clone(),
            },
            &n2.node_info,
        )
        .await;

    println!("res5: {:?}", res5);

    let _ = tokio::join!(rh1, ph1, remh1, rh2, ph2, remh2);
}
