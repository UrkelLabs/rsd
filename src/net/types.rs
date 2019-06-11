use std::net::SocketAddr;
use std::time::SystemTime;

/// Default protocol version
pub const PROTOCOL_VERSION: u32 = 1;

/// Minimum protocol version that we'll talk to
pub const MIN_PROTOCOL_VERSION: u32 = 1;

/// User agent passed along in messages
pub const USER_AGENT: &str = concat!("RSD:", env!("CARGO_PKG_VERSION"));

///Maximum message size that can be sent ~8mb
pub const MAX_MESSAGE_SIZE: u32 = 8_000_000;

///Amount of time to ban misbehaving peers
pub const BAN_TIME: u32 = 86_400;

///Ban score threshold before ban is placed in effect
pub const BAN_SCORE: u32 = 100;

///Maximum inv/getdata size
pub const MAX_INV: u32 = 50_000;

///Maximum number of requests
pub const MAX_REQUEST: u32 = 5_000;

///Maximum number of block requests
pub const MAX_BLOCK_REQUEST: u32 = 50_000 + 1_000;

///Maximum number of transaction requests
pub const MAX_TX_REQUEST: u32 = 10_000;

///Maximum number of claim requests
pub const MAX_CLAIM_REQUEST: u32 = 1_000;

/// Service constant for Network capabilities (1 << 0)
const NETWORK: u64 = 1;

///Service constant for Bloom Filter capabilities
const BLOOM: u64 = (1 << 1);

///Service definition for a full node - (3)
const FULL_NODE: u64 = NETWORK | BLOOM;

///Service definition required to communicate - (3)
const REQUIRED_SERVICES: u64 = NETWORK | BLOOM;

///Service definition for rsd - (3)
const LOCAL_SERVICES: u64 = NETWORK | BLOOM;

//I think this has to be a
//33 byte array, but let's double check this.
//Also I think we should base this off Buffer?
//TODO new type called SetBuffer? Which has a preset length.
pub struct IdentityKey([u8; 33]);

//Service Enum
pub enum Services {
    None,
    Network,
    Bloom,
    FullNode,
    RequiredServices,
    LocalServices,
}

impl Services {
    pub fn value(&self) -> u64 {
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
    address: SocketAddr,
    services: Services,
    //TODO check type on this.
    time: SystemTime,
    key: IdentityKey,
}

impl PeerAddr {
    //TODO probably include services in the new function, instead of setting it to 0.
    pub fn new(addr: SocketAddr, key: IdentityKey) -> PeerAddr {
        PeerAddr {
            address: addr,
            key,
            time: SystemTime::now(),
            //Init as none, can change later.
            services: Services::None,
        }
    }
}

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
