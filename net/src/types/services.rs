use crate::common::{BLOOM, FULL_NODE, NETWORK};
use crate::error;
use std::convert::TryFrom;

//Service Enum
#[derive(Debug, Clone, Copy)]
pub enum Services {
    Unknown,
    Network,
    Bloom,
    FullNode,
    // RequiredServices,
    // LocalServices,
}

impl TryFrom<u32> for Services {
    type Error = error::Error;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        let service = match value {
            0 => Services::Unknown,
            NETWORK => Services::Network,
            BLOOM => Services::Bloom,
            // FULL_NODE => Services::FullNode,
            // REQUIRED_SERVICES =>, Services::RequiredServices,
            // LOCAL_SERVICES =>
            _ => return Err(error::Error::UnknownService),
        };

        Ok(service)
    }
}

impl Services {
    pub fn value(&self) -> u32 {
        match *self {
            Services::Unknown => 0,
            //1
            Services::Network => NETWORK,
            //2
            Services::Bloom => BLOOM,
            Services::FullNode => FULL_NODE,
            // Services::RequiredServices => REQUIRED_SERVICES,
            // Services::LocalServices => LOCAL_SERVICES,
        }
    }
}
