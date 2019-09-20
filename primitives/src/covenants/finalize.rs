use extended_primitives::{Buffer, Hash, Uint256, VarInt};
use handshake_encoding::{Decodable, DecodingError, Encodable};
use handshake_types::{Name, NameHash};

//@todo formatting, and I think common functions to_hex, from_hex.
//@todo testing.
//@when I say formatting I mean Debug and to_string functions.

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct FinalizeCovenant {
    pub name_hash: NameHash,
    pub height: u32,
    pub name: Name,
    //TODO see above.
    pub flags: String,
    //TODO see above
    pub block_hash: Hash,
}

impl Encodable for FinalizeCovenant {
    fn size(&self) -> usize {
        let mut size = 0;
        let name_hash_length = VarInt::from(32 as u64);
        let height_length = VarInt::from(4 as u64);
        let name_length = VarInt::from(self.name.len() as u64);
        let flags_length = VarInt::from(self.flags.len() as u64);
        let block_hash_length = VarInt::from(32 as u64);

        size += name_hash_length.encoded_size() as usize;
        size += height_length.encoded_size() as usize;
        size += name_length.encoded_size() as usize;
        size += flags_length.encoded_size() as usize;
        size += block_hash_length.encoded_size() as usize;

        size
    }

    fn encode(&self) -> Buffer {
        let mut buffer = Buffer::new();

        buffer.write_u8(10);
        buffer.write_varint(5);

        //Name Hash
        //Hashes are 32 bytes
        buffer.write_varint(32);
        buffer.write_hash(self.name_hash);

        //Height
        buffer.write_varint(4);
        buffer.write_u32(self.height);

        //Record Data
        buffer.write_varint(self.name.len());
        buffer.write_str(&self.name);

        //Record Data
        buffer.write_varint(self.flags.len());
        buffer.write_str(&self.flags);

        //Block Hash
        buffer.write_varint(32);
        buffer.write_hash(self.block_hash);

        buffer
    }
}

impl Decodable for FinalizeCovenant {
    type Err = DecodingError;

    fn decode(buffer: &mut Buffer) -> Result<Self, Self::Err> {
        //5
        buffer.read_varint()?;

        buffer.read_varint()?;
        let name_hash = buffer.read_hash()?;

        buffer.read_varint()?;
        let height = buffer.read_u32()?;

        //Name
        let name_length = buffer.read_varint()?;
        let name = buffer.read_string(name_length.as_u64() as usize)?;

        //Flags
        let flags_length = buffer.read_varint()?;
        let flags = buffer.read_string(flags_length.as_u64() as usize)?;

        buffer.read_varint()?;
        let block_hash = buffer.read_hash()?;

        Ok(FinalizeCovenant {
            name_hash,
            height,
            name: Name::from(name),
            flags,
            block_hash,
        })
    }
}
