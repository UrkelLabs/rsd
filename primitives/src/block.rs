use crate::BlockHeader;
use crate::Transaction;
use extended_primitives::Buffer;
use handshake_protocol::encoding::{Decodable, DecodingError, Encodable};

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
    fn size(&self) -> u32 {
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
    type Error = DecodingError;

    fn decode(buffer: &mut Buffer) -> Result<Self, Self::Error> {
        let header = BlockHeader::decode(buffer)?;

        let count = buffer.read_varint()?;
        let mut txdata = Vec::new();

        for _ in 0..count.as_u64() {
            txdata.push(Transaction::decode(buffer)?);
        }

        Ok(Block { header, txdata })
    }
}
