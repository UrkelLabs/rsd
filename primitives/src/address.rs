use bech32::u5;
use extended_primitives::{Buffer, BufferError};
use handshake_protocol::encoding::{Decodable, DecodingError, Encodable};
use std::str::FromStr;

//TODO possibly move this to just a file error.
#[derive(Debug)]
enum AddressError {
    InvalidAddressVersion,
    InvalidAddressSize,
    InvalidNetworkPrefix,
    InvalidHash,
    Decoding(DecodingError),
    Bech32(bech32::Error),
    Buffer(BufferError),
}

impl From<DecodingError> for AddressError {
    fn from(e: DecodingError) -> Self {
        AddressError::Decoding(e)
    }
}

impl From<bech32::Error> for AddressError {
    fn from(e: bech32::Error) -> Self {
        AddressError::Bech32(e)
    }
}

impl From<BufferError> for AddressError {
    fn from(e: BufferError) -> Self {
        AddressError::Buffer(e)
    }
}

#[derive(PartialEq, Clone, Debug)]
enum Payload {
    PubkeyHash(Buffer),
    ScriptHash(Buffer),
}

impl Payload {
    fn len(&self) -> usize {
        match self {
            Payload::PubkeyHash(hash) => hash.len(),
            Payload::ScriptHash(hash) => hash.len(),
        }
    }

    fn to_hash(self) -> Buffer {
        match self {
            Payload::PubkeyHash(hash) => hash,
            Payload::ScriptHash(hash) => hash,
        }
    }

    fn from_hash(hash: Buffer) -> Result<Payload, AddressError> {
        match hash.len() {
            20 => Ok(Payload::PubkeyHash(hash)),
            32 => Ok(Payload::ScriptHash(hash)),
            _ => Err(AddressError::InvalidHash),
        }
    }
}

#[derive(PartialEq, Clone, Debug)]
pub struct Address {
    //Can we make this u8? TODO
    //And do we even need this?
    pub version: u8,
    pub hash: Payload,
}

impl Address {
    //TODO
    // pub fn is_null(&self) -> bool {
    //     self.hash.is_null()
    // }

    pub fn is_null_data(&self) -> bool {
        self.version == 31
    }

    pub fn is_unspendable(&self) -> bool {
        self.is_null_data()
    }
}

impl Decodable for Address {
    type Error = AddressError;

    fn decode(buffer: &mut Buffer) -> Result<Self, Self::Error> {
        let version = buffer.read_u8()?;

        if version > 31 {
            return Err(AddressError::InvalidAddressVersion);
        }

        let size = buffer.read_u8()?;

        if size < 2 || size > 40 {
            return Err(AddressError::InvalidAddressSize);
        }

        let hash = buffer.read_bytes(size as usize)?;

        let hash = Payload::from_hash(Buffer::from(hash))?;

        Ok(Address {
            version: version,
            hash,
        })
    }
}

impl Encodable for Address {
    fn size(&self) -> u32 {
        1 + 1 + self.hash.len() as u32
    }

    fn encode(&self) -> Buffer {
        let mut buffer = Buffer::new();

        buffer.write_u8(self.version);
        buffer.write_u8(self.hash.len() as u8);
        buffer.extend(self.hash.to_hash());

        buffer
    }
}

impl FromStr for Address {
    type Err = AddressError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (hrp, data) = bech32::decode(s)?;

        let (version, hash) = version_hash_from_bech32(data);

        let hash = Payload::from_hash(hash)?;

        Ok(Address { version, hash })
    }
}

// //TODO eq, partial eq, ordering.

fn version_hash_from_bech32(data: Vec<u5>) -> (u8, Buffer) {
    let version = data[0].to_u8();
    let mut hash = Buffer::new();

    let iter = data.iter();

    for (i, elem) in iter.enumerate() {
        if i == 0 {
            continue;
        }

        hash.write_u8(elem.to_u8());
    }

    (version, hash)
}
