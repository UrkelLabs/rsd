use crate::net::common::{
    BLOOM, FULL_NODE, LOCAL_SERVICES, NETWORK, PROTOCOL_VERSION, REQUIRED_SERVICES,
};
use crate::protocol::encoding::Encodable;
use crate::types::Buffer;
use base32;
use chrono::{DateTime, Utc};
use hex;
use std::fmt;
use std::net::SocketAddr;
use std::ops;
use std::str::FromStr;
use std::time::SystemTime;

//TODO new type called SetBuffer? Which has a preset length.
//TODO extended primitives Buffer with capacity.
#[derive(Clone)]
pub struct IdentityKey([u8; 33]);

impl IdentityKey {
    pub fn as_array(self) -> [u8; 33] {
        self.0
    }
}

impl From<[u8; 33]> for IdentityKey {
    fn from(key: [u8; 33]) -> Self {
        IdentityKey(key)
    }
}

impl ops::Deref for IdentityKey {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl AsRef<[u8]> for IdentityKey {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl AsMut<[u8]> for IdentityKey {
    fn as_mut(&mut self) -> &mut [u8] {
        &mut self.0
    }
}

impl FromStr for IdentityKey {
    //TODO wrap all errors here
    type Err = hex::FromHexError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.len() {
            64 => {
                let mut bytes = [0; 33];
                let bytes_slice = hex::decode(s)?;

                bytes.copy_from_slice(&bytes_slice);

                Ok(IdentityKey(bytes))
            }
            53 => {
                //TODO use a ? here, and then map it to the above error
                let key = base32::decode(base32::Alphabet::RFC4648 { padding: false }, s).unwrap();

                let mut bytes = [0; 33];
                bytes.copy_from_slice(&key);

                Ok(IdentityKey(bytes))
            }
            _ => Err(hex::FromHexError::InvalidStringLength),
        }
    }
}

impl fmt::Debug for IdentityKey {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "IdentityKey: {}", hex::encode(self.0.to_vec()))
    }
}

//Service Enum
#[derive(Debug, Clone)]
pub enum Services {
    None,
    Network,
    Bloom,
    FullNode,
    RequiredServices,
    LocalServices,
}

impl Services {
    pub fn value(&self) -> u32 {
        match *self {
            Services::None => 0,
            //1
            Services::Network => NETWORK,
            //2
            Services::Bloom => BLOOM,
            Services::FullNode => FULL_NODE,
            Services::RequiredServices => REQUIRED_SERVICES,
            Services::LocalServices => LOCAL_SERVICES,
        }
    }
}

//TODO maybe move this to it's own file.
//TODO I think tear down SocketAddr and store more raw
#[derive(Clone, Debug)]
pub struct PeerAddr {
    pub address: SocketAddr,
    pub services: Services,
    //TODO check type on this.
    pub time: DateTime<Utc>,
    pub key: IdentityKey,
}

impl PeerAddr {
    //TODO probably include services in the new function, instead of setting it to 0.
    pub fn new(addr: SocketAddr, key: IdentityKey) -> PeerAddr {
        PeerAddr {
            address: addr,
            key,
            time: Utc::now(),
            //Init as none, can change later.
            services: Services::None,
        }
    }
}

//PeerAddr impl ToString

impl Encodable for PeerAddr {
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

//add default -> Include default services in there.

//TODO from string for PeerAddr

#[derive(Clone, Copy, Debug, Eq, Ord, PartialOrd, PartialEq)]
pub struct ProtocolVersion(pub u32);

impl Default for ProtocolVersion {
    fn default() -> ProtocolVersion {
        ProtocolVersion(PROTOCOL_VERSION)
    }
}

impl From<ProtocolVersion> for u32 {
    fn from(v: ProtocolVersion) -> u32 {
        v.0
    }
}
