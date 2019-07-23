use crate::Stack;
use extended_primitives::Buffer;
use handshake_protocol::encoding::{Decodable, DecodingError, Encodable};

pub struct Witness {
    //TODO probably not u8
    stack: Stack<Buffer>,
}

impl Encodable for Witness {
    //Does not include the varint of total items on the stack.
    fn size(&self) -> u32 {
        //TODO
        32
    }

    fn encode(&self) -> Buffer {
        let mut buffer = Buffer::new();

        buffer.write_varint(self.stack.len());

        let stack = self.stack.iter();

        for item in stack {
            buffer.write_var_bytes(item);
        }

        buffer
    }
}

impl Decodable for Witness {
    type Error = DecodingError;

    fn decode(buffer: &mut Buffer) -> Result<Self, Self::Error> {
        let count = buffer.read_varint()?;

        //if count.as_u64() > consensus.max_script_stack() {
        //    //too many witness items error
        //    Script error.
        //}

        let mut stack = Stack::new();
        for _ in 0..count.as_u64() {
            let item = Buffer::from(buffer.read_var_bytes()?);
            stack.push(item);
        }

        Ok(Witness { stack })
    }
}
