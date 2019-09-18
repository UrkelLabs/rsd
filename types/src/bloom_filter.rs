use extended_primitives::{Buffer, VarInt};
use handshake_encoding::{Decodable, DecodingError, Encodable};
use pds::BloomFilter;

//Expose the filter flags, as well as encoding and decoding here.
//
//TODO revamp doc comments
#[derive(Copy, Clone, PartialEq, Debug)]
pub enum BloomFlags {
    // Don't update filter w/ outpoints.
    None = 0,
    // Always update filter w/ outpoints.
    All = 1,
    //Only update filter w/ outpoints if "asymmetric" in terms of addresses (pubkey/multisig).
    PubKeyOnly = 2,
}

impl BloomFlags {
    fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(BloomFlags::None),
            1 => Some(BloomFlags::All),
            2 => Some(BloomFlags::PubKeyOnly),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Bloom {
    //TODO pub here?
    pub filter: BloomFilter,
    pub flag: BloomFlags,
}

//TODO wrap all functions from BloomFilter

impl Encodable for Bloom {
    //TODO really need to double check this.
    fn size(&self) -> usize {
        let mut size = 0;
        let length = VarInt::from(self.filter.filter.len() as u64);
        size += length.encoded_size() as usize;
        size += self.filter.filter.len();
        size += 9;
        size
    }

    fn encode(&self) -> Buffer {
        let mut buffer = Buffer::new();

        buffer.write_var_bytes(&self.filter.filter);
        buffer.write_u32(self.filter.n);
        buffer.write_u32(self.filter.tweak);
        buffer.write_u8(self.flag as u8);

        buffer
    }
}

impl Decodable for Bloom {
    type Err = DecodingError;

    fn decode(buffer: &mut Buffer) -> Result<Self, Self::Err> {
        let filter = buffer.read_var_bytes()?;

        let n = buffer.read_u32()?;
        let tweak = buffer.read_u32()?;
        let flag = buffer.read_u8()?;

        let bloom_filter = BloomFilter::new_with_filter(filter, n, tweak);

        if let Some(flag) = BloomFlags::from_u8(flag) {
            Ok(Bloom {
                filter: bloom_filter,
                flag,
            })
        } else {
            Err(DecodingError::InvalidData(
                "Invalid Bloom Filter Flag".to_owned(),
            ))
        }
    }
}
