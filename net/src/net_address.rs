use crate::error;
use crate::types::{IdentityKey, Services};
// use crate::Result;
use chrono::{DateTime, TimeZone, Utc};
use extended_primitives::Buffer;
use handshake_protocol::encoding::{Decodable, Encodable};
use std::convert::TryFrom;
use std::net::{IpAddr, SocketAddr};

//TODO I think tear down SocketAddr and store more raw
#[derive(Clone, Debug, Copy)]
pub struct NetAddress {
    pub address: SocketAddr,
    pub services: Services,
    pub time: DateTime<Utc>,
    pub key: IdentityKey,
}

impl NetAddress {
    pub fn new(addr: SocketAddr, key: IdentityKey) -> Self {
        NetAddress {
            address: addr,
            key,
            time: Utc::now(),
            //Init as none, can change later.
            services: Services::UNKNOWN,
        }
    }
}

impl Encodable for NetAddress {
    fn size(&self) -> u32 {
        88
    }

    fn encode(&self) -> Buffer {
        let mut buffer = Buffer::new();

        buffer.write_u64(self.time.timestamp() as u64);
        buffer.write_u32(self.services.bits());
        buffer.write_u32(0);
        buffer.write_u8(0);
        match self.address.ip() {
            IpAddr::V4(ip) => buffer.write_bytes(&ip.to_ipv6_mapped().octets()),
            IpAddr::V6(ip) => buffer.write_bytes(&ip.octets()),
        }
        buffer.fill(0, 20);
        buffer.write_u16(self.address.port());
        buffer.write_bytes(&self.key);

        buffer
    }
}

impl Decodable for NetAddress {
    type Error = error::Error;

    fn decode(mut buf: Buffer) -> Result<NetAddress, Self::Error> {
        //Don't like this -> See if we should just make our own time type that wraps this.
        let timestamp = Utc.timestamp(buf.read_u64()? as i64, 0);
        let services = Services::from_bits_truncate(buf.read_u32()?);
        let ip: String;

        buf.read_u32()?;

        if buf.read_u8()? == 0 {
            ip = buf.read_string(16)?;
            buf.seek(20)?;
        } else {
            //Ugly don't do this, but I don't see us ever hitting this loop.
            ip = "0000000000000000".to_owned();
            buf.seek(36)?;
        }

        let port = buf.read_u16()?;
        //Convert this to read_fixed_bytes then we don't need to use try_from
        let key = IdentityKey::try_from(buf.read_bytes(33)?)?;

        let hostname = format!("{}:{}", ip, port);

        Ok(NetAddress {
            address: hostname.parse()?,
            key,
            time: timestamp,
            services,
        })
    }
}

// impl FromStr for NetAddress {

// }

//PeerAddr impl ToString
//add default -> Include default services in there.
//TODO from string for PeerAddr
//
//TODO make net it's own package in fact, let's make everything it's own package.
//

mod test {
    use super::*;

    #[test]
    fn test_net_address_encoding() {
        let hostname = "127.0.0.1:8333";
        let key = IdentityKey::from([0; 33]);

        let mut addr = NetAddress::new(hostname.parse().unwrap(), key);
    }

}
