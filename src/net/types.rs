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

//TODO I think tear down SocketAddr and store more raw
pub struct PeerAddr {
    address: SocketAddr,
    services: Services,
    //TODO check type on this.
    time: SystemTime,
    key: IdentityKey,
}

//I think this has to be a
//33 byte array, but let's double check this.
//Also I think we should base this off Buffer?
//TODO new type called SetBuffer? Which has a preset length.
pub struct IdentityKey([u8; 33]);

//Service Enum
pub enum Services {
    Network,
    Bloom,
    FullNode,
    RequiredServices,
    LocalServices,
}

impl Services {
    pub fn value(&self) -> u64 {
        match *self {
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
