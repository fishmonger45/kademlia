# kademlia
learning implementation
# todo
- change serde_json to bincode or whatever is the fastest struct encoder
- fancy repl with fancy display for each node
- anyhow + error propigation (especially from async!)
- split cli and library into seperate crates, publish on crates.io when done
- also do integration tests in lib.rs
# resources
- [short explaination on leading zeros as a distance metric](https://stackoverflow.com/questions/48602172/how-to-represent-kademlia-distance-metric-as-integer)
- [udp datagram payload size](https://stackoverflow.com/questions/1098897/what-is-the-largest-safe-udp-packet-size-on-the-internet/35697810#35697810)