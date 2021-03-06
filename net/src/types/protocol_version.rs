use crate::common::PROTOCOL_VERSION;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialOrd, PartialEq)]
pub struct ProtocolVersion(pub u32);

//Probably a better way to do this.
impl ProtocolVersion {
    pub fn as_u32(&self) -> u32 {
        self.0
    }
}

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

impl From<u32> for ProtocolVersion {
    fn from(v: u32) -> Self {
        ProtocolVersion(v)
    }
}

//TODO make this an enum? meh maybe not. but we need a value to know if this doesn't exist already.
