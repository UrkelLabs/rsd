use extended_primitives::{Buffer, BufferError};
use std::fmt;

#[derive(Debug)]
pub enum DecodingError {
    Buffer(BufferError),
    UnknownInvetory,
}

impl From<BufferError> for DecodingError {
    fn from(e: BufferError) -> DecodingError {
        DecodingError::Buffer(e)
    }
}

impl fmt::Display for DecodingError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            DecodingError::Buffer(ref e) => write!(f, "Buffer Error: {}", e),
            DecodingError::UnknownInvetory => write!(f, "Unknown Inventory Type"),
        }
    }
}

pub trait Encodable {
    fn size(&self) -> u32;

    fn encode(&self) -> Buffer;
}

pub trait Decodable
where
    Self: Sized,
{
    type Error;
    fn decode(buffer: &mut Buffer) -> Result<Self, Self::Error>;
}
