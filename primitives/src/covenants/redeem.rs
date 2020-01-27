use extended_primitives::{Buffer, VarInt};
use handshake_encoding::{Decodable, DecodingError, Encodable};
use handshake_types::NameHash;

//@todo formatting, and I think common functions to_hex, from_hex.
//@todo testing.
//@when I say formatting I mean Debug and to_string functions.
//
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct RedeemCovenant {
    pub name_hash: NameHash,
    pub height: u32,
}

impl Encodable for RedeemCovenant {
    fn size(&self) -> usize {
        let mut size = VarInt::from(2 as u64).encoded_size() as usize;
        let name_hash_length = VarInt::from(32 as u64);
        let height_length = VarInt::from(4 as u64);

        size += name_hash_length.encoded_size() as usize;
        size += height_length.encoded_size() as usize;

        size
    }

    fn encode(&self) -> Buffer {
        let mut buffer = Buffer::new();

        buffer.write_u8(5);
        buffer.write_varint(2);

        //Name Hash
        //Hashes are 32 bytes
        buffer.write_varint(32);
        buffer.write_hash(self.name_hash);

        //Height
        buffer.write_varint(4);
        buffer.write_u32(self.height);

        buffer
    }
}

impl Decodable for RedeemCovenant {
    type Err = DecodingError;

    fn decode(buffer: &mut Buffer) -> Result<Self, Self::Err> {
        //2
        buffer.read_varint()?;

        buffer.read_varint()?;
        let name_hash = buffer.read_hash()?;

        buffer.read_varint()?;
        let height = buffer.read_u32()?;

        Ok(RedeemCovenant { name_hash, height })
    }
}
