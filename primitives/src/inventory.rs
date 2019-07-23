use extended_primitives::{Buffer, Hash};
use handshake_protocol::encoding::{Decodable, DecodingError, Encodable};

#[derive(Copy, Clone, Debug, PartialEq)]
enum InvType {
    Tx = 1,
    Block = 2,
    FilteredBlock = 3,
    CompactBlock = 4,
    Claim = 5,
    Airdrop = 6,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Inventory {
    _type: InvType,
    hash: Hash,
}

impl Inventory {
    pub fn is_block(&self) -> bool {
        match self._type {
            InvType::Block => true,
            InvType::CompactBlock => true,
            InvType::FilteredBlock => true,
            _ => false,
        }
    }

    pub fn is_tx(&self) -> bool {
        match self._type {
            InvType::Tx => true,
            _ => false,
        }
    }

    pub fn is_claim(&self) -> bool {
        match self._type {
            InvType::Claim => true,
            _ => false,
        }
    }

    pub fn is_airdrop(&self) -> bool {
        match self._type {
            InvType::Airdrop => true,
            _ => false,
        }
    }
}

impl Encodable for Inventory {
    fn size(&self) -> u32 {
        36
    }

    fn encode(&self) -> Buffer {
        let mut buffer = Buffer::new();

        buffer.write_u32(self._type as u32);
        buffer.write_hash(self.hash);

        buffer
    }
}

impl Decodable for Inventory {
    type Error = DecodingError;

    fn decode(buffer: &mut Buffer) -> Result<Self, Self::Error> {
        let num_type = buffer.read_u32()?;

        //Probably wrap this into a function TODO
        let _type = match num_type {
            1 => InvType::Tx,
            2 => InvType::Block,
            3 => InvType::FilteredBlock,
            4 => InvType::CompactBlock,
            5 => InvType::Claim,
            6 => InvType::Airdrop,
            _ => return Err(DecodingError::UnknownInventory),
        };
        let hash = buffer.read_hash()?;

        Ok(Inventory { _type, hash })
    }
}
