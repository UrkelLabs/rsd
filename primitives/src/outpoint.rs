use extended_primitives::{Buffer, Hash};
use handshake_encoding::{Decodable, DecodingError, Encodable};
use std::fmt;

//TODO should we impl Odr?
#[derive(Copy, Clone, Eq, PartialEq)]
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
    fn size(&self) -> usize {
        //32 (txid) + 4 (index)
        36
    }

    fn encode(&self) -> Buffer {
        let mut buffer = Buffer::new();

        buffer.write_hash(self.txid);
        buffer.write_u32(self.index);

        buffer
    }
}

impl Decodable for Outpoint {
    type Err = DecodingError;

    fn decode(buffer: &mut Buffer) -> Result<Self, Self::Err> {
        let txid = buffer.read_hash()?;
        let index = buffer.read_u32()?;

        Ok(Outpoint { txid, index })
    }
}

//@todo Testing
//@todo test compare and Ord - ensure it matches hsd
//@todo JSON Serialization
//@todo impl From<TX>

impl fmt::Display for Outpoint {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<Outpoint: {}/{}>", self.txid, self.index)
    }
}

impl fmt::Debug for Outpoint {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<Outpoint: {}/{}>", self.txid, self.index)
    }
}
