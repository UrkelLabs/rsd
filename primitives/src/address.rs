use bech32::u5;
use extended_primitives::Buffer;
use handshake_encoding::{Decodable, DecodingError, Encodable};
use std::str::FromStr;

#[derive(Debug)]
pub enum AddressError {
    InvalidAddressVersion,
    InvalidAddressSize,
    InvalidNetworkPrefix,
    InvalidHash,
    Bech32(bech32::Error),
}

impl From<bech32::Error> for AddressError {
    fn from(e: bech32::Error) -> Self {
        AddressError::Bech32(e)
    }
}

impl From<AddressError> for DecodingError {
    fn from(e: AddressError) -> DecodingError {
        DecodingError::InvalidData(format!("{:?}", e))
    }
}

// #[derive(PartialEq, Clone, Debug, Copy)]
#[derive(PartialEq, Clone, Debug)]
pub enum Payload {
    PubkeyHash(Buffer),
    ScriptHash(Buffer),
}

impl Payload {
    pub fn len(&self) -> usize {
        match self {
            Payload::PubkeyHash(hash) => hash.len(),
            Payload::ScriptHash(hash) => hash.len(),
        }
    }

    pub fn to_hash(self) -> Buffer {
        match self {
            Payload::PubkeyHash(hash) => hash,
            Payload::ScriptHash(hash) => hash,
        }
    }

    pub fn as_hash(&self) -> &Buffer {
        match self {
            Payload::PubkeyHash(hash) => hash,
            Payload::ScriptHash(hash) => hash,
        }
    }

    pub fn from_hash(hash: Buffer) -> Result<Payload, AddressError> {
        match hash.len() {
            20 => Ok(Payload::PubkeyHash(hash)),
            32 => Ok(Payload::ScriptHash(hash)),
            _ => Err(AddressError::InvalidHash),
        }
    }
}

//@todo ideally implement copy here, but we need to implement it for Buffer, and we really need to
//look into performance degration there.
// #[derive(PartialEq, Clone, Debug, Copy)]
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

//@todo review if this is a good default. Should be triggered on "null"
impl Default for Address {
    fn default() -> Self {
        Address {
            version: 0,
            hash: Payload::PubkeyHash(Buffer::new()),
        }
    }
}

impl Decodable for Address {
    type Err = DecodingError;

    fn decode(buffer: &mut Buffer) -> Result<Self, Self::Err> {
        let version = buffer.read_u8()?;

        if version > 31 {
            return Err(DecodingError::InvalidData(
                "Invalid Address Version".to_string(),
            ));
        }

        let size = buffer.read_u8()?;

        if size < 2 || size > 40 {
            return Err(DecodingError::InvalidData(
                "Invalid Address Size".to_string(),
            ));
        }

        let hash = buffer.read_bytes(size as usize)?;

        let hash = Payload::from_hash(Buffer::from(hash))?;

        Ok(Address { version, hash })
    }
}

impl Encodable for Address {
    fn size(&self) -> usize {
        1 + 1 + self.hash.len()
    }

    fn encode(&self) -> Buffer {
        let mut buffer = Buffer::new();

        buffer.write_u8(self.version);
        buffer.write_u8(self.hash.len() as u8);
        //TODO fix this
        buffer.extend(self.hash.as_hash().clone());

        buffer
    }
}

impl FromStr for Address {
    type Err = AddressError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        //@todo should we be checking network here?
        let (_hrp, data) = bech32::decode(s)?;

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
