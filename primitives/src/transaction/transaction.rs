use crate::{Input, Output};
use extended_primitives::{Buffer, Hash};
use handshake_encoding::{Decodable, DecodingError, Encodable};
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
    pub fn new(locktime: u32, inputs: Vec<Input>, outputs: Vec<Output>) -> Self {
        Transaction {
            //@todo not sure what exactly what we should be doing here.
            version: 0,
            locktime,
            inputs,
            outputs,
        }
    }

    //TODO implement - see hsd primitives
    //we should keep similar behavior with hash and witness hash
    pub fn hash(&self) -> Hash {
        Default::default()
    }

    //@todo
    pub fn witness_hash(&self) -> Hash {
        Default::default()
    }
}

impl Encodable for Transaction {
    fn size(&self) -> usize {
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
            buffer.extend(input.witness.encode())
        }

        buffer
    }
}

impl Decodable for Transaction {
    type Err = DecodingError;

    fn decode(buffer: &mut Buffer) -> Result<Self, Self::Err> {
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
            inputs[i as usize].witness = witness;
        }

        Ok(Transaction {
            version,
            locktime,
            inputs,
            outputs,
        })
    }
}
