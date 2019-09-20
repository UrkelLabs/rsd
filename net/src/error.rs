use brontide;
use extended_primitives;
use handshake_protocol;
use hex;
use std::fmt;
use std::net::AddrParseError;

//TODO implement errors for each subtype we have in this package.
//So PoolError, PeerError, etc, etc
//Then wrap them all up in Error enum.

#[derive(Debug)]
pub enum Error {
    Brontide(brontide::Error),
    Buffer(extended_primitives::BufferError),
    Decoding(handshake_protocol::encoding::DecodingError),
    InvalidHostname(AddrParseError),
    Hex(hex::FromHexError),
    FutureIO(futures::io::Error),
    LockError,
    Base32,
    InvalidIdentityKey,
    InvalidNetAddress,
    DuplicateVersion,
}

impl From<brontide::Error> for Error {
    fn from(e: brontide::Error) -> Self {
        Error::Brontide(e)
    }
}

impl From<futures::io::Error> for Error {
    fn from(e: futures::io::Error) -> Self {
        Error::FutureIO(e)
    }
}

impl From<handshake_protocol::encoding::DecodingError> for Error {
    fn from(e: handshake_protocol::encoding::DecodingError) -> Self {
        Error::Decoding(e)
    }
}

impl From<extended_primitives::BufferError> for Error {
    fn from(e: extended_primitives::BufferError) -> Self {
        Error::Buffer(e)
    }
}

impl From<hex::FromHexError> for Error {
    fn from(e: hex::FromHexError) -> Self {
        Error::Hex(e)
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
            Error::Decoding(ref e) => write!(f, "Encoding error: {}", e),
            Error::InvalidHostname(ref e) => write!(f, "Invalid Hostname error: {}", e),
            Error::FutureIO(ref e) => write!(f, "Futures IO error: {}", e),
            Error::Hex(ref e) => write!(f, "Hex error: {}", e),
            Error::Base32 => write!(f, "Base32 error"),
            Error::InvalidIdentityKey => write!(f, "Invalid Identity Key"),
            Error::InvalidNetAddress => write!(f, "Invalid Network Address"),
            Error::DuplicateVersion => write!(f, "Peer sent a duplicate version."),
            Error::LockError => write!(f, "Lock error"),
        }
    }
}
