use extended_primitives::{Buffer, Hash, VarInt};
use handshake_encoding::{Decodable, DecodingError, Encodable};
use handshake_types::{Name, NameHash};

//@todo formatting, and I think common functions to_hex, from_hex.
//when I say formatting I mean Debug and to_string functions.

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ClaimCovenant {
    pub name_hash: NameHash,
    pub height: u32,
    pub name: Name,
    //@todo would love to see an Enum here.
    pub flags: u8,
    pub commit_hash: Hash,
    pub commit_height: u32,
}

impl ClaimCovenant {
    pub fn get_items(&self) -> Vec<Buffer> {
        let mut items = Vec::new();

        let mut buffer = Buffer::new();
        buffer.write_hash(self.name_hash);
        items.push(buffer);

        let mut buffer = Buffer::new();
        buffer.write_u32(self.height);
        items.push(buffer);

        let mut buffer = Buffer::new();
        buffer.write_str(&self.name);
        items.push(buffer);

        let mut buffer = Buffer::new();
        buffer.write_u8(self.flags);
        items.push(buffer);

        let mut buffer = Buffer::new();
        buffer.write_hash(self.commit_hash);
        items.push(buffer);

        let mut buffer = Buffer::new();
        buffer.write_u32(self.commit_height);
        items.push(buffer);

        items
    }

    //@todo this should really return a result.
    pub fn from_items(mut items: Vec<Buffer>) -> ClaimCovenant {
        let name_len = items[2].len();
        let name_hash = items[0].read_hash().unwrap();
        let height = items[1].read_u32().unwrap();
        let name = items[2].read_string(name_len).unwrap();
        let flags = items[3].read_u8().unwrap();
        let commit_hash = items[4].read_hash().unwrap();
        let commit_height = items[5].read_u32().unwrap();

        ClaimCovenant {
            name_hash,
            height,
            name: name.parse().unwrap(),
            flags,
            commit_hash,
            commit_height,
        }
    }
}

impl Encodable for ClaimCovenant {
    fn size(&self) -> usize {
        let mut size = VarInt::from(6 as u64).encoded_size() as usize;
        let name_hash_length = VarInt::from(32 as u64);
        let height_length = VarInt::from(4 as u64);
        let name_length = VarInt::from(self.name.len() as u64);
        let flags_length = VarInt::from(1 as u64);
        let commit_hash_length = VarInt::from(32 as u64);
        let commit_height_length = VarInt::from(4 as u64);

        size += name_hash_length.encoded_size() as usize;
        size += height_length.encoded_size() as usize;
        size += name_length.encoded_size() as usize;
        size += flags_length.encoded_size() as usize;
        size += commit_hash_length.encoded_size() as usize;
        size += commit_height_length.encoded_size() as usize;
        size += 32;
        size += 4;
        size += self.name.len();
        size += 1;
        size += 32;
        size += 4;

        size
    }

    fn encode(&self) -> Buffer {
        let mut buffer = Buffer::new();

        buffer.write_u8(1);
        buffer.write_varint(6);

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

        //Flags
        buffer.write_varint(1);
        buffer.write_u8(self.flags);

        //Commit Hash
        buffer.write_varint(32);
        buffer.write_hash(self.commit_hash);

        //Commit Height
        buffer.write_varint(4);
        buffer.write_u32(self.commit_height);

        buffer
    }
}

impl Decodable for ClaimCovenant {
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
        let flags = buffer.read_u8()?;

        buffer.read_varint()?;
        let commit_hash = buffer.read_hash()?;

        buffer.read_varint()?;
        let commit_height = buffer.read_u32()?;

        Ok(ClaimCovenant {
            name_hash,
            height,
            name: name.parse().unwrap(),
            flags,
            commit_hash,
            commit_height,
        })
    }
}
