use extended_primitives::{Buffer, Hash, VarInt};
use handshake_encoding::{Decodable, DecodingError, Encodable};
use handshake_types::NameHash;

//@todo formatting, and I think common functions to_hex, from_hex.
//@todo testing.
//@when I say formatting I mean Debug and to_string functions.

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct RegisterCovenant {
    pub name_hash: NameHash,
    pub height: u32,
    //TODO verify type. I believe this is a serialized encoding of the data insert. We should make
    //this a cutom type.
    pub record_data: String,
    //TODO not sure if we want to have BlockHash custom type, but it might make sense.
    //Otherwise Hash here is just fine
    pub block_hash: Hash,
}

impl Encodable for RegisterCovenant {
    fn size(&self) -> usize {
        let mut size = 0;
        let name_hash_length = VarInt::from(32 as u64);
        let height_length = VarInt::from(4 as u64);
        let name_length = VarInt::from(self.record_data.len() as u64);
        let block_length = VarInt::from(32 as u64);

        //@todo fix.
        size += name_hash_length.encoded_size() as usize;
        size += height_length.encoded_size() as usize;
        size += name_length.encoded_size() as usize;
        size += block_length.encoded_size() as usize;

        size
    }

    fn encode(&self) -> Buffer {
        let mut buffer = Buffer::new();

        buffer.write_u8(6);
        buffer.write_varint(4);

        //Name Hash
        //Hashes are 32 bytes
        buffer.write_varint(32);
        buffer.write_hash(self.name_hash);

        //Height
        buffer.write_varint(4);
        buffer.write_u32(self.height);

        //Record Data
        buffer.write_varint(self.record_data.len());
        buffer.write_str(&self.record_data);

        //Block Hash
        buffer.write_varint(32);
        buffer.write_hash(self.block_hash);

        buffer
    }
}

impl Decodable for RegisterCovenant {
    type Err = DecodingError;

    fn decode(buffer: &mut Buffer) -> Result<Self, Self::Err> {
        //4
        buffer.read_varint()?;

        buffer.read_varint()?;
        let name_hash = buffer.read_hash()?;

        buffer.read_varint()?;
        let height = buffer.read_u32()?;

        //Record Data
        let record_length = buffer.read_varint()?;
        let record_data = buffer.read_string(record_length.as_u64() as usize)?;

        buffer.read_varint()?;
        let block_hash = buffer.read_hash()?;

        Ok(RegisterCovenant {
            name_hash,
            height,
            record_data,
            block_hash,
        })
    }
}
