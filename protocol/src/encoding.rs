use handshake_types::Buffer;

pub trait Encodable {
    fn size() -> u32;

    fn encode(&self) -> Buffer;
}
