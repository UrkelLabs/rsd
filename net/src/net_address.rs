use crate::types::{IdentityKey, Services};
use chrono::{DateTime, Utc};
use handshake_protocol::encoding::Encodable;
use handshake_types::Buffer;
use std::net::SocketAddr;
use std::str::FromStr;

//TODO I think tear down SocketAddr and store more raw
#[derive(Clone, Debug, Copy)]
pub struct NetAddress {
    pub address: SocketAddr,
    pub services: Services,
    pub time: DateTime<Utc>,
    pub key: IdentityKey,
}

impl NetAddress {
    //TODO probably include services in the new function, instead of setting it to 0.
    pub fn new(addr: SocketAddr, key: IdentityKey) -> Self {
        NetAddress {
            address: addr,
            key,
            time: Utc::now(),
            //Init as none, can change later.
            services: Services::None,
        }
    }
}

impl Encodable for NetAddress {
    fn size() -> u32 {
        88
    }

    fn encode(&self) -> Buffer {
        let mut buffer = Buffer::new();

        buffer.write_u64(self.time.timestamp() as u64);
        buffer.write_u32(self.services.value());
        buffer.write_u32(0);
        buffer.write_u8(0);
        buffer.write_string(self.address.ip().to_string());
        buffer.fill(0, 20);
        buffer.write_u16(self.address.port());
        buffer.write_bytes(&self.key);

        buffer
    }
}

// impl FromStr for NetAddress {

// }

//PeerAddr impl ToString
//add default -> Include default services in there.
//TODO from string for PeerAddr
//
//TODO make net it's own package in fact, let's make everything it's own package.
