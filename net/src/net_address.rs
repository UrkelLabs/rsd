use crate::error;
use crate::types::{IdentityKey, Services};
// use crate::Result;
use chrono::{DateTime, TimeZone, Utc};
use extended_primitives::Buffer;
use handshake_protocol::encoding::{Decodable, Encodable};
use std::convert::TryFrom;
use std::net::{IpAddr, SocketAddr};
use std::str::FromStr;

//TODO I think tear down SocketAddr and store more raw
#[derive(Clone, Debug, Copy, PartialEq)]
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
            //Wrap time into our own type because this will become troublesome. TODO
            time: Utc.timestamp(Utc::now().timestamp(), 0),
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

    fn decode(buf: &mut Buffer) -> Result<NetAddress, Self::Error> {
        //Don't like this -> See if we should just make our own time type that wraps this.
        let timestamp = Utc.timestamp(buf.read_u64()? as i64, 0);
        let services = Services::from_bits_truncate(buf.read_u32()?);
        let mut ip_bytes = [0; 16];

        buf.read_u32()?;

        if buf.read_u8()? == 0 {
            //Make this read_exact_bytes TODO
            let bytes = buf.read_bytes(16)?;
            ip_bytes.copy_from_slice(&bytes);
            buf.seek(20)?;
        } else {
            //Ugly don't do this, but I don't see us ever hitting this loop.
            // ip = "0000000000000000".to_owned();
            ip_bytes = [0; 16];
            buf.seek(36)?;
        }

        let port = buf.read_u16()?;
        //Convert this to read_fixed_bytes then we don't need to use try_from
        let key = IdentityKey::try_from(buf.read_bytes(33)?)?;

        let ip = IpAddr::from(ip_bytes);
        let is_v4 = match ip {
            IpAddr::V4(ip) => Some(ip),
            IpAddr::V6(ip) => ip.to_ipv4(),
        };

        let hostname: SocketAddr;

        if let Some(ipv4) = is_v4 {
            let addr = IpAddr::from(ipv4);
            hostname = SocketAddr::new(addr, port);
        } else {
            hostname = SocketAddr::new(ip, port);
        }

        Ok(NetAddress {
            address: hostname,
            key,
            time: timestamp,
            services,
        })
    }
}

impl FromStr for NetAddress {
    type Err = error::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let pieces: Vec<&str> = s.split('@').collect();

        if pieces.len() != 2 {
            return Err(error::Error::InvalidIdentityKey);
        }

        let key: IdentityKey = pieces[0].parse()?;

        let address: SocketAddr = pieces[1].parse()?;

        Ok(NetAddress {
            address,
            key,
            time: Utc.timestamp(Utc::now().timestamp(), 0),
            services: Services::UNKNOWN,
        })
    }
}

//PeerAddr impl ToString
//add default -> Include default services in there.
//TODO from string for PeerAddr
//

mod test {
    use super::*;

    #[test]
    fn test_net_address_encode_and_decode() {
        let hostname = "127.0.0.1:8333";
        let key = IdentityKey::from([0; 33]);

        let addr = NetAddress::new(hostname.parse().unwrap(), key);

        let addr2 = NetAddress::decode(&mut addr.encode()).unwrap();

        dbg!(addr);

        assert_eq!(addr, addr2);
    }

    #[test]
    fn test_net_address_encoding() {
        let hostname = "127.0.0.1:8333";
        let key = IdentityKey::from([0; 33]);

        let mut addr = NetAddress::new(hostname.parse().unwrap(), key);
        addr.time = Utc.timestamp(1231006505, 0);
        addr.services = Services::NETWORK;

        assert_eq!(addr.encode().into_hex(), "29ab5f490000000001000000000000000000000000000000000000ffff7f00000100000000000000000000000000000000000000008d20000000000000000000000000000000000000000000000000000000000000000000");
    }

}
