use handshake_types::Buffer;

pub trait Encodable {
    fn size(&self) -> u32;

    fn encode(&self) -> Buffer;
}
