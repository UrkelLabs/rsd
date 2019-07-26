use extended_primitives::{Buffer, VarInt};
use handshake_protocol::encoding::{Decodable, DecodingError, Encodable};
use pds::BloomFilter;

//Expose the filter flags, as well as encoding and decoding here.
//
//TODO revamp doc comments
pub enum BloomFlags {
    // Don't update filter w/ outpoints.
    None = 0,
    // Always update filter w/ outpoints.
    All = 1,
    //Only update filter w/ outpoints if "asymmetric" in terms of addresses (pubkey/multisig).
    PubKeyOnly = 2,
}

pub struct Bloom {
    //TODO pub here?
    pub filter: BloomFilter,
    pub flag: BloomFlags,
}

impl Encodable for Bloom {
    //TODO really need to double check this.
    fn size(&self) -> u32 {
        let mut size = 0;
        let length = VarInt::from(self.filter.filter.len() as u64);
        size += length.encoded_size();
        size += self.filter.filter.len() as u32;
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

//TODO need to expose Bloom_filter from new w/ Filter.
// impl Decodable for Bloom {
//     type Error = DecodingError;

//     fn decode(&mut buffer: Buffer) -> Result<Self, Self::Error> {

//         let filter

//     }
// }
