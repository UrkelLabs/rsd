use crate::Transaction;
use extended_primitives::{Buffer, Hash};
use handshake_encoding::{Decodable, DecodingError, Encodable};
use std::default::Default;
use std::fmt;

//@todo should we impl Odr?
//@todo Eq and PartialEq probably need to be rewritten for BIP69
#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub struct Outpoint {
    txid: Hash,
    index: u32,
}

impl Outpoint {
    pub fn new(hash: Hash, index: u32) -> Outpoint {
        Outpoint { txid: hash, index }
    }

    pub fn is_null(&self) -> bool {
        *self == Outpoint::default()
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

// ===== From Implementations =====

impl From<(Transaction, u32)> for Outpoint {
    fn from((tx, index): (Transaction, u32)) -> Self {
        Outpoint {
            txid: tx.hash(),
            index,
        }
    }
}

// ===== Default =====

impl Default for Outpoint {
    fn default() -> Outpoint {
        Outpoint {
            txid: Hash::default(),
            index: u32::max_value(),
        }
    }
}

// ===== Display and Debug =====

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
