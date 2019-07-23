use crate::{Address, Covenant};
use extended_primitives::Buffer;
use handshake_protocol::encoding::{Decodable, DecodingError, Encodable};
use handshake_types::Amount;

#[derive(Clone, PartialEq, Debug)]
pub struct Output {
    value: Amount,
    address: Address,
    covenant: Covenant,
}

//TODO get size, is_dust, format, equal + peq, to hex from hex, to buffer, from buffer.
impl Output {
    pub fn is_unspendable(&self) -> bool {
        self.address.is_unspendable() | self.covenant.is_unspendable()
    }
}

impl Encodable for Output {
    fn size(&self) -> u32 {
        8 + self.address.size() + self.covenant.size()
    }

    fn encode(&self) -> Buffer {
        let mut buffer = Buffer::new();

        buffer.write_u64(self.value.as_doos());
        buffer.extend(self.address.encode());
        buffer.extend(self.covenant.encode());

        buffer
    }
}

impl Decodable for Output {
    type Error = DecodingError;

    fn decode(buffer: &mut Buffer) -> Result<Self, Self::Error> {
        //TODO make from doos
        let value = Amount::from_doo(buffer.read_u64()?);
        let address = Address::decode(buffer)?;
        let covenant = Covenant::decode(buffer)?;

        Ok(Output {
            value,
            address,
            covenant,
        })
    }
}
