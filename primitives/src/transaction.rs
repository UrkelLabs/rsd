use crate::{Input, Output};
use extended_primitives::{Buffer, Hash};
use handshake_protocol::encoding::{Decodable, DecodingError, Encodable};
use handshake_script::Witness;

#[derive(Clone, PartialEq, Debug)]
pub struct Transaction {
    /// The protocol version, is currently expected to be 1 or 2 (BIP 68).
    pub version: u32,
    /// Block number before which this transaction is valid, or 0 for
    /// valid immediately.
    pub locktime: u32,
    pub inputs: Vec<Input>,
    pub outputs: Vec<Output>,
}

impl Transaction {
    //TODO implement - see hsd primitives
    //we should keep similar behavior with hash and witness hash
    pub fn hash(&self) -> Hash {
        Default::default()
    }
}

impl Encodable for Transaction {
    fn size(&self) -> u32 {
        //TODO
        32
    }

    fn encode(&self) -> Buffer {
        let mut buffer = Buffer::new();

        buffer.write_u32(self.version);
        buffer.write_varint(self.inputs.len());

        for input in self.inputs.iter() {
            buffer.extend(input.encode());
        }

        buffer.write_varint(self.outputs.len());

        for output in self.outputs.iter() {
            buffer.extend(output.encode());
        }

        buffer.write_u32(self.locktime);

        for input in self.inputs.iter() {
            //Probably not the most efficient way to do this, TODO review this code.
            if let Some(witness) = input.witness {
                buffer.extend(witness.encode());
            }
        }

        buffer
    }
}

impl Decodable for Transaction {
    type Error = DecodingError;

    fn decode(buffer: &mut Buffer) -> Result<Self, Self::Error> {
        let mut inputs = Vec::new();
        let mut outputs = Vec::new();

        let version = buffer.read_u32()?;

        let input_count = buffer.read_varint()?;
        for _ in 0..input_count.as_u64() {
            inputs.push(Input::decode(buffer)?);
        }

        let output_count = buffer.read_varint()?;
        for _ in 0..output_count.as_u64() {
            outputs.push(Output::decode(buffer)?);
        }

        let locktime = buffer.read_u32()?;

        //TODO varint as usize please.
        for i in 0..input_count.as_u64() {
            let witness = Witness::decode(buffer)?;
            inputs[i as usize].witness = Some(witness);
        }

        Ok(Transaction {
            version,
            locktime,
            inputs,
            outputs,
        })
    }
}
