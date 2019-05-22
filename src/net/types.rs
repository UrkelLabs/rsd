use std::net::SocketAddr;
use std::time::SystemTime;

//TODO I think tear down SocketAddr and store more raw
pub struct PeerAddr {
    address: SocketAddr,

    //TODO make this a custom type.
    services: u32,
    //TODO check type on this.
    time: SystemTime,
    key: IdentityKey,
}

//I think this has to be a
//33 byte array, but let's double check this.
//Also I think we should base this off Buffer?
//TODO new type called SetBuffer? Which has a preset length.
pub struct IdentityKey([u8; 33]);

pub enum Services {
    Network,
    Bloom,
    FullNode,
    RequiredServices,
}

//Service constants
const NETWORK: u64 = (1 << 0);
const BLOOM: u64 = (1 << 1);
//I don't think the 0 is needed here, let's double check that.
const FULL_NODE: u64 = 0 | NETWORK | BLOOM;
const REQUIRED_SERVICES: u64 = 0 | NETWORK | BLOOM;

impl Services {
    pub fn value(&self) -> u64 {
        match *self {
            //1
            Services::Network => NETWORK,
            //2
            Services::Bloom => BLOOM,
            Services::FullNode => FULL_NODE,
            Services::RequiredServices => REQUIRED_SERVICES,
        }
    }
}
