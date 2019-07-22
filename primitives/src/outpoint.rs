use extended_primitives::{Buffer, Hash};
use handshake_protocol::encoding::{Decodable, DecodingError, Encodable};

//TODO should we impl Odr?
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct Outpoint {
    txid: Hash,
    index: u32,
}

impl Outpoint {
    ///Returns a null Outpoint for use in coinbase transactions.
    pub fn null() -> Outpoint {
        Outpoint {
            txid: Default::default(),
            index: u32::max_value(),
        }
    }

    pub fn is_null(&self) -> bool {
        *self == Outpoint::null()
    }
}

impl Encodable for Outpoint {
    fn size(&self) -> u32 {
        //32 (txid) + 4 (index)
        36
    }

    fn encode(&self) -> Buffer {
        let buffer = Buffer::new();

        buffer.write_hash(self.txid);
        buffer.write_u32(self.index);

        buffer
    }
}

impl Decodable for Outpoint {
    type Error = DecodingError;

    fn decode(buffer: &mut Buffer) -> Result<Self, Self::Error> {
        let txid = buffer.read_hash()?;
        let index = buffer.read_u32()?;

        Ok(Outpoint { txid, index })
    }
}
