use crate::types::Buffer;

pub trait Encodable {
    //TODO might not need to take self.
    fn size(&self) -> u32;

    fn encode(&self) -> Buffer;
}
