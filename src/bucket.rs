use std::net::IpAddr;

use crate::id::Id;

pub struct Info{
    address: IpAddr,
    // port: 
    id: Id,
}

pub type Bucket = Vec<Info>;