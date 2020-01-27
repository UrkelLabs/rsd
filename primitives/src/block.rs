use crate::BlockHeader;
use crate::Transaction;
use encodings::hex::{FromHex, ToHex};
use extended_primitives::Buffer;
use handshake_encoding::{Decodable, DecodingError, Encodable};

/// A Handshake block, which is a collection of transactions with an attached
/// proof of work.
#[derive(PartialEq, Clone, Debug)]
pub struct Block {
    /// The block header
    pub header: BlockHeader,
    /// List of transactions contained in the block
    pub txdata: Vec<Transaction>,
}

impl Encodable for Block {
    fn size(&self) -> usize {
        //TODO relies on tx's get size which is not done.
        32
    }

    fn encode(&self) -> Buffer {
        let mut buffer = Buffer::new();

        buffer.extend(self.header.encode());

        buffer.write_varint(self.txdata.len());

        for tx in self.txdata.iter() {
            buffer.extend(tx.encode());
        }

        buffer
    }
}

impl Decodable for Block {
    type Err = DecodingError;

    fn decode(buffer: &mut Buffer) -> Result<Self, Self::Err> {
        let header = BlockHeader::decode(buffer)?;

        let count = buffer.read_varint()?;
        let mut txdata = Vec::new();

        for _ in 0..count.as_u64() {
            txdata.push(Transaction::decode(buffer)?);
        }

        Ok(Block { header, txdata })
    }
}

impl ToHex for Block {
    fn to_hex(&self) -> String {
        self.encode().to_hex()
    }
}

impl FromHex for Block {
    type Error = DecodingError;

    fn from_hex<T: AsRef<[u8]>>(hex: T) -> std::result::Result<Self, Self::Error> {
        Block::decode(&mut Buffer::from_hex(hex)?)
    }
}
