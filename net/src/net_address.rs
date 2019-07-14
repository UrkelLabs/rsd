use crate::error;
use crate::types::{IdentityKey, RawIP, Services};
// use crate::Result;
use extended_primitives::Buffer;
use handshake_protocol::encoding::{Decodable, Encodable};
use handshake_types::Time;
use std::convert::TryFrom;
use std::net::{IpAddr, SocketAddr};
use std::str::FromStr;

//TODO I think tear down SocketAddr and store more raw
#[derive(Clone, Debug, Copy)]
pub struct NetAddress {
    //TODO remove this and add a to_socketaddr
    pub address: SocketAddr,
    pub raw: RawIP,
    pub port: u16,
    pub services: Services,
    pub time: Time,
    pub key: IdentityKey,
}

impl NetAddress {
    pub fn new(addr: SocketAddr, key: IdentityKey) -> Self {
        NetAddress {
            address: addr,
            raw: RawIP::from(addr.ip()),
            port: addr.port(),
            key,
            time: Time::new(),
            services: Services::UNKNOWN,
        }
    }

    //Return the unqiue key that represents this network address
    pub fn get_unique_key(&self) -> [u8; 18] {
        //TODO do we include some reference to the identity key here?
        let mut key = [0_u8; 18];

        key.copy_from_slice(&self.raw);
        key[16] = (self.port / 0x100) as u8; // most significant byte of our port
        key[17] = (self.port / 0x0FF) as u8; // least significant byte of our port

        key
    }

    pub fn get_group(&self) -> Vec<u8> {
        let mut group = Vec::new();

        //IPV6
        let mut class = 2;
        let mut start_byte = 0;
        let mut bits = 16;

        // all local addresses belong to the same group
        if self.raw.is_local() {
            class = 255;
            bits = 0;
        };

        if !self.raw.is_routable() {
            //Unroutable
            class = 0;
            bits = 0;
        // all other unroutable addresses belong to the same group
        } else if self.raw.is_ipv4() || self.raw.is_rfc6145() || self.raw.is_rfc6052() {
            //IPV4
            class = 1;
            start_byte = 12;
        } else if self.raw.is_rfc3964() {
            //IPV4
            class = 1;
            start_byte = 2;
        } else if self.raw.is_rfc4380() {
            //IPV4
            group.push(1);
            group.push(self.raw[12] ^ 0xFF);
            group.push(self.raw[13] ^ 0xFF);
            return group;
        } else if self.raw.is_onion() {
            //Onion
            class = 3;
            start_byte = 6;
            bits = 4;
        // for he.net, use /36 groups
        } else if self.raw[0] == 0x20
            && self.raw[1] == 0x01
            && self.raw[2] == 0x04
            && self.raw[3] == 0x70
        {
            bits = 36;
        } else {
            bits = 32;
        }

        group.push(class);
        while bits >= 8 {
            group.push(self.raw[start_byte as usize]);
            start_byte += 1;
            bits -= 8;
        }

        if bits > 0 {
            group.push(self.raw[start_byte as usize] | ((1 << (8 - bits)) - 1));
        }

        group
    }

    pub fn is_valid(&self) -> bool {
        //TODO probably also check if the identity key is not null.
        //Although, I don't think a Netaddress will ever be constructed without one, so this should
        //be fine. Just a note that we probably still should check for self.key.is_null() something
        //like that.
        self.raw.is_valid()
    }
}

//TODO we have to decide if we want to make Peers unique on
//identity keys + ip or if we just want to use ip.
impl std::hash::Hash for NetAddress {
    /// If loopback address then we care about ip and port.
    /// If regular address then we only care about the ip and ignore the port.

    //TODO decide if we want to hash by key or not.
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        if self.address.ip().is_loopback() {
            //Hashes ip + port
            self.address.hash(state);
        } else {
            //Hashes only ip
            self.address.ip().hash(state);
        }
    }
}

//TODO also decide if this uses key or not.
impl PartialEq for NetAddress {
    /// If loopback address then we care about ip and port.
    /// If regular address then we only care about the ip and ignore the port.
    fn eq(&self, other: &NetAddress) -> bool {
        if self.address.ip().is_loopback() {
            self.address == other.address
        } else {
            self.address.ip() == other.address.ip()
        }
    }
}

impl Eq for NetAddress {}

impl Encodable for NetAddress {
    fn size(&self) -> u32 {
        88
    }

    fn encode(&self) -> Buffer {
        let mut buffer = Buffer::new();

        buffer.write_u64(self.time.to_seconds());
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
        let timestamp = Time::from(buf.read_u64()?);
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
            raw: RawIP::from(hostname.ip()),
            port: hostname.port(),
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
            raw: RawIP::from(address.ip()),
            port: address.port(),
            key,
            time: Time::new(),
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
        addr.time = Time::from(1231006505);
        addr.services = Services::NETWORK;

        assert_eq!(addr.encode().into_hex(), "29ab5f490000000001000000000000000000000000000000000000ffff7f00000100000000000000000000000000000000000000008d20000000000000000000000000000000000000000000000000000000000000000000");
    }

    #[test]
    fn test_net_address_get_group() {
        //Local -> Unroutable
        let hostname = "127.0.0.1:0000";
        let addr = NetAddress::new(hostname.parse().unwrap(), [0; 33].into());
        assert_eq!(addr.get_group(), vec![0]);

        //RFC1918 -> Unroutable
        let hostname = "10.0.0.1:0000";
        let addr = NetAddress::new(hostname.parse().unwrap(), [0; 33].into());
        assert_eq!(addr.get_group(), vec![0]);

        //RFC3927 -> Unroutable
        let hostname = "169.254.1.1:0000";
        let addr = NetAddress::new(hostname.parse().unwrap(), [0; 33].into());
        assert_eq!(addr.get_group(), vec![0]);

        //IPv4
        let hostname = "1.2.3.4:0000";
        let addr = NetAddress::new(hostname.parse().unwrap(), [0; 33].into());
        assert_eq!(addr.get_group(), vec![1, 1, 2]);

        //RFC6145
        let hostname = "[::FFFF:0:102:304]:0000";
        let addr = NetAddress::new(hostname.parse().unwrap(), [0; 33].into());
        assert_eq!(addr.get_group(), vec![1, 1, 2]);

        //RFC6052
        let hostname = "[64:FF9B::102:304]:0000";
        let addr = NetAddress::new(hostname.parse().unwrap(), [0; 33].into());
        assert_eq!(addr.get_group(), vec![1, 1, 2]);

        //RFC3964
        let hostname = "[2002:102:304:9999:9999:9999:9999:9999]:0000";
        let addr = NetAddress::new(hostname.parse().unwrap(), [0; 33].into());
        assert_eq!(addr.get_group(), vec![1, 1, 2]);

        //RFC4380
        let hostname = "[2001:0:9999:9999:9999:9999:FEFD:FCFB]:0000";
        let addr = NetAddress::new(hostname.parse().unwrap(), [0; 33].into());
        assert_eq!(addr.get_group(), vec![1, 1, 2]);

        //Onion
        let hostname = "[FD87:D87E:EB43:edb1:8e4:3588:e546:35ca]:0000";
        let addr = NetAddress::new(hostname.parse().unwrap(), [0; 33].into());
        assert_eq!(addr.get_group(), vec![3, 239]);

        //he.net
        let hostname = "[2001:470:abcd:9999:9999:9999:9999:9999]:0000";
        let addr = NetAddress::new(hostname.parse().unwrap(), [0; 33].into());
        assert_eq!(addr.get_group(), vec![2, 32, 1, 4, 112, 175]);

        //IPv6
        let hostname = "[2001:2001:9999:9999:9999:9999:9999:9999]:0000";
        let addr = NetAddress::new(hostname.parse().unwrap(), [0; 33].into());
        assert_eq!(addr.get_group(), vec![2, 32, 1, 32, 1]);
    }

}
