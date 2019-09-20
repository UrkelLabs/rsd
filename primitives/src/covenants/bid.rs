use extended_primitives::{Buffer, Hash, VarInt};
use handshake_encoding::{Decodable, DecodingError, Encodable};
use handshake_types::{Name, NameHash};

//@todo formatting, and I think common functions to_hex, from_hex.
//@todo testing.
//@when I say formatting I mean Debug and to_string functions.

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct BidCovenant {
    pub name_hash: NameHash,
    pub height: u32,
    pub name: Name,
    //TODO *might* want to make this a BidHash, but that'll be a later impl
    pub hash: Hash,
}

impl Encodable for BidCovenant {
    fn size(&self) -> usize {
        let mut size = 0;
        let name_hash_length = VarInt::from(32 as u64);
        let height_length = VarInt::from(4 as u64);
        let name_length = VarInt::from(self.name.len() as u64);
        let hash_length = VarInt::from(32 as u64);

        //@todo I think we should switch varint encoded size to usize
        size += name_hash_length.encoded_size() as usize;
        size += height_length.encoded_size() as usize;
        size += name_length.encoded_size() as usize;
        size += hash_length.encoded_size() as usize;

        size
    }

    fn encode(&self) -> Buffer {
        let mut buffer = Buffer::new();

        buffer.write_u8(3);
        buffer.write_varint(4);

        //Name Hash
        //Hashes are 32 bytes
        buffer.write_varint(32);
        buffer.write_hash(self.name_hash);

        //Height
        buffer.write_varint(4);
        buffer.write_u32(self.height);

        //Name
        buffer.write_varint(self.name.len());
        buffer.write_str(&self.name);

        //Hashes are 32 bytes
        buffer.write_varint(32);
        buffer.write_hash(self.hash);

        buffer
    }
}

impl Decodable for BidCovenant {
    type Err = DecodingError;

    fn decode(buffer: &mut Buffer) -> Result<Self, Self::Err> {
        //4
        buffer.read_varint()?;

        buffer.read_varint()?;
        let name_hash = buffer.read_hash()?;

        buffer.read_varint()?;
        let height = buffer.read_u32()?;

        let name_length = buffer.read_varint()?;
        //TODO check
        let name = buffer.read_string(name_length.as_u64() as usize)?;

        buffer.read_varint()?;
        let hash = buffer.read_hash()?;

        Ok(BidCovenant {
            name_hash,
            height,
            name: Name::from(name),
            hash,
        })
    }
}
