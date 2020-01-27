use crate::address::Payload;
use crate::Address;
use extended_primitives::{Buffer, VarInt};
use handshake_encoding::{Decodable, DecodingError, Encodable};
use handshake_types::NameHash;

//@todo formatting, and I think common functions to_hex, from_hex.
//@todo testing.
//@when I say formatting I mean Debug and to_string functions.

#[derive(Debug, Clone, PartialEq)]
pub struct TransferCovenant {
    pub name_hash: NameHash,
    pub height: u32,
    pub version: u8,
    pub address: Address,
}

impl Encodable for TransferCovenant {
    fn size(&self) -> usize {
        let mut size = VarInt::from(4 as u64).encoded_size() as usize;
        //TODO because all these values are below 252
        let name_hash_length = VarInt::from(32 as u64);
        let height_length = VarInt::from(4 as u64);
        let version_length = VarInt::from(1 as u64);
        let address_length = VarInt::from(self.address.hash.as_hash().len() as u64);

        size += name_hash_length.encoded_size() as usize;
        size += height_length.encoded_size() as usize;
        size += version_length.encoded_size() as usize;
        size += address_length.encoded_size() as usize;

        size
    }

    fn encode(&self) -> Buffer {
        let mut buffer = Buffer::new();

        buffer.write_u8(9);
        buffer.write_varint(4);

        //Name Hash
        //Hashes are 32 bytes
        buffer.write_varint(32);
        buffer.write_hash(self.name_hash);

        //Height
        buffer.write_varint(4);
        buffer.write_u32(self.height);

        //Version
        buffer.write_varint(1);
        buffer.write_u8(self.version);

        //Block Hash
        buffer.write_var_bytes(self.address.hash.as_hash());

        buffer
    }
}

impl Decodable for TransferCovenant {
    type Err = DecodingError;

    fn decode(buffer: &mut Buffer) -> Result<Self, Self::Err> {
        //4
        buffer.read_varint()?;

        buffer.read_varint()?;
        let name_hash = buffer.read_hash()?;

        buffer.read_varint()?;
        let height = buffer.read_u32()?;

        buffer.read_varint()?;
        let version = buffer.read_u8()?;

        buffer.read_varint()?;
        let hash = Buffer::from(buffer.read_var_bytes()?);
        //@todo not sure if I'm a fan of this. Maybe just keep it as a addr_hash in the struct.
        let address = Address {
            version: 0,
            hash: Payload::from_hash(hash).unwrap(),
        };

        Ok(TransferCovenant {
            name_hash,
            height,
            version,
            address,
        })
    }
}
