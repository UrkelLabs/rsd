use bech32::u5;
use extended_primitives::{Buffer, Hash};
use handshake_protocol::encoding::{Decodable, DecodingError, Encodable};
use handshake_protocol::network::Network;
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

#[derive(PartialEq, Clone, Debug)]
enum Payload {
    PubkeyHash(Buffer),
    ScriptHash(Buffer),
}

impl Payload {
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
    pub version: u32,
    pub hash: Payload,
    pub network: Network,
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

    //pub fn from_hash(hash: Buffer, version: Option<u8>) -> Result<Address, AddressError> {
    //    let hash_version = match version {
    //        Some(version) => version,
    //        None => 0,
    //    };

    //    //TODO should we actually wrap all these inside of the payload enum?
    //    //Then we can reuse for read.
    //    if hash_version > 31 {
    //        return Err(AddressError::InvalidAddressVersion);
    //    } else if hash.len() < 2 || hash.len() > 40 {
    //        return Err(AddressError::InvalidAddressSize);
    //    }

    //    //TODO double check logic.
    //    if hash_version == 0 && !(hash.len() == 20 || hash.len() == 32) {
    //        return Err(AddressError::InvalidAddressSize);
    //    }

    //    // this.hash = hash;
    //    // this.version = version;

    //    // return this;
    //    // }
    //}
}

//    this.hash = hash;
//    this.version = version;

//    return this;
//  }

// impl Decodable for Address {
//     type Error = AddressError;

//     fn decode(buffer: &mut Buffer) -> Result<Self, Self::Error> {
//         let version = buffer.read_u8()?;

//         if version > 31 {
//             return Err(AddressError::InvalidAddressVersion);
//         }

//         let size = buffer.read_u8()?;

//         if size < 2 || size > 40 {
//             return Err(AddressError::InvalidAddressSize);
//         }

//         let hash = buffer.read_bytes(size as usize);
//     }
// }
// impl From<String> for Address {
//     fn from(item: i32) -> Self {
//         Address { value: item }
//     }
// }
//
// //TODO from string, eq, partial eq, ordering.
impl FromStr for Address {
    type Err = AddressError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (hrp, data) = bech32::decode(s)?;

        let (version, hash) = version_hash_from_bech32(data);

        let hash = Payload::from_hash(hash)?;

        let network = match Network::from_bech32_prefix(&hrp) {
            Some(network) => network,
            None => return Err(AddressError::InvalidNetworkPrefix),
        };

        Ok(Address {
            version,
            hash,
            network,
        })
    }
}

fn version_hash_from_bech32(data: Vec<u5>) -> (u32, Buffer) {
    let version = data[0].to_u8();
    let mut hash = Buffer::new();

    let iter = data.iter();

    for (i, elem) in iter.enumerate() {
        if i == 0 {
            continue;
        }

        hash.write_u8(elem.to_u8());
    }

    (version as u32, hash)
}
