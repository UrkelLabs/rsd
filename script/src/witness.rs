use crate::Stack;
use extended_primitives::{Buffer, VarInt};
use handshake_encoding::{Decodable, DecodingError, Encodable};

//@todo fromStr
//@todo toString
//@todo Display and Debug manual
//@todo json serialization
//@todo consider moving witness to primitives
//@todo actually consider moving the entire script package to primitives.

#[derive(Clone, PartialEq, Debug)]
pub struct Witness {
    pub stack: Stack<Buffer>,
}

impl Witness {
    pub fn new() -> Self {
        Witness {
            stack: Stack::new(),
        }
    }

    pub fn push_data(&mut self, data: Buffer) {
        self.stack.push(data);
    }

    pub fn set_data(&mut self, index: usize, data: Buffer) {
        self.stack.set(index, data);
    }

    pub fn var_size(&self) -> usize {
        let varint = VarInt::from(self.stack.len());
        varint.encoded_size() as usize + self.size()
    }
}

impl Encodable for Witness {
    //Does not include the varint of total items on the stack.
    fn size(&self) -> usize {
        let mut size = 0;

        for item in self.stack.iter() {
            let varint = VarInt::from(item.len());
            size += varint.encoded_size() as usize + item.len();
        }

        size
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
    type Err = DecodingError;

    fn decode(buffer: &mut Buffer) -> Result<Self, Self::Err> {
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
