use std::{error::Error, io::Write, sync::Arc};

use tokio::sync::Mutex;

use crate::{id::Id, node::Node};

/// Tracked `Runtime`
pub struct Runtime {
    nodes: Arc<Mutex<Vec<Arc<Node>>>>,
    selected: Option<Arc<Node>>,
}

impl Runtime {
    /// Create a new runtime tracker
    pub fn new() -> Self {
        Self {
            nodes: Arc::new(Mutex::new(Vec::new())),
            selected: None,
        }
    }

    pub async fn spawn(&mut self, ip: String, port: String) -> Result<(), Box<dyn Error>> {
        let node = Arc::new(Node::new(format!("{}:{}", ip, port)).await?);
        let nodes = Arc::clone(&self.nodes);
        tokio::spawn(async move {
            let (h1, h2, h3) = node.start();
            println!("{node}");
            {
                let mut nodes = nodes.lock().await;
                nodes.push(Arc::clone(&node));
            }
            let _ = tokio::join!(h1, h2, h3);
        });

        Ok(())
    }
    /// List active nodes
    pub async fn list(&self) {
        let nodes = self.nodes.lock().await;
        let mut sep = ' ';
        for n in nodes.iter() {
            // if self.selected.is_some() && self.selected.as_ref().unwrap() == &n.node_info.id {
            //     sep = 'x';
            // }
            println!("[{}] {}", sep, n.node_info.id.hex());
        }
    }

    /// Start REPL
    pub async fn start(&mut self) -> Result<(), Box<dyn Error>> {
        let mut buffer = String::new();
        loop {
            print!("> ");
            std::io::stdout().flush()?;
            buffer.clear();
            std::io::stdin().read_line(&mut buffer).unwrap();
            let args: Vec<&str> = buffer.trim().split(" ").collect();
            match args[0] {
                // Spawn a new node
                "" => continue,
                "spawn" => {
                    if args.len() != 3 {
                        Self::help();
                        continue;
                    }

                    let ip = args[1].to_string();
                    let port = args[2].to_string();
                    self.spawn(ip, port).await?;
                }
                // List all spawned nodes, with an x next to the one that you have selected
                "list" => {
                    self.list().await;
                }
                // Switch to using a different node
                "switch" => {}
                "ping" => {}
                "find" => {}
                "get" => {}
                "" => {}
                "history" => {}
                "help" => {}
                iv => {
                    println!("< invalid command \"{iv}\"");
                }
            }
        }
    }

    async fn select(&mut self, id: String) {
        let id = Id::from(id.as_str());
        let nodes = self.nodes.lock().await;
        for n in nodes.iter() {
            if n.node_info.id == id {
                self.selected = Some(Arc::clone(n));
                return;
            }
        }
        println!("Unable to find node id");
    }

    /// Print the help dialog
    fn help() {
        println!(r#""#)
    }
}
