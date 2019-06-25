use brontide;
use extended_primitives;
use handshake_protocol;
use std::fmt;
use std::net::AddrParseError;

#[derive(Debug)]
pub enum Error {
    Brontide(brontide::Error),
    Buffer(extended_primitives::BufferError),
    Encoding(handshake_protocol::encoding::Error),
    InvalidHostname(AddrParseError),
    UnknownService,
    InvalidIdentityKey,
}

impl From<brontide::Error> for Error {
    fn from(e: brontide::Error) -> Self {
        Error::Brontide(e)
    }
}

impl From<handshake_protocol::encoding::Error> for Error {
    fn from(e: handshake_protocol::encoding::Error) -> Self {
        Error::Encoding(e)
    }
}

impl From<extended_primitives::BufferError> for Error {
    fn from(e: extended_primitives::BufferError) -> Self {
        Error::Buffer(e)
    }
}

impl From<AddrParseError> for Error {
    fn from(e: AddrParseError) -> Self {
        Error::InvalidHostname(e)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::Brontide(ref e) => write!(f, "Brontide error: {}", e),
            Error::Buffer(ref e) => write!(f, "Buffer error: {}", e),
        }
    }
}
