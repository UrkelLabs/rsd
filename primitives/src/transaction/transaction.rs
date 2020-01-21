use crate::{Input, Output};
use cryptoxide::blake2b::Blake2b;
use cryptoxide::digest::Digest;
use extended_primitives::{Buffer, Hash, VarInt};
use handshake_encoding::{Decodable, DecodingError, Encodable};
use handshake_script::Witness;
use hex::{FromHex, FromHexError};

#[derive(Clone, PartialEq, Debug, Default)]
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

    pub fn hash(&self) -> Hash {
        let raw = self.encode();

        let base_size = self.get_base_size();

        let base = raw[..base_size].to_vec();

        let mut sh = Blake2b::new(32);
        let mut output = [0; 32];
        sh.input(&base);
        sh.result(&mut output);

        Hash::from(output)
    }

    pub fn witness_hash(&self) -> Hash {
        let raw = self.encode();

        let base_size = self.get_base_size();

        let witness = raw[base_size..].to_vec();

        let mut sh = Blake2b::new(32);
        let mut output = [0; 32];
        sh.input(&witness);
        sh.result(&mut output);

        let mut final_output = [0; 32];
        let mut sh = Blake2b::new(32);
        sh.input(&self.hash().to_array());
        sh.input(&output);
        sh.result(&mut final_output);

        Hash::from(final_output)
    }

    pub fn is_null(&self) -> bool {
        *self == Default::default()
    }

    pub fn get_base_size(&self) -> usize {
        let mut size = 0;
        size += 4;

        size += VarInt::from(self.inputs.len()).encoded_size() as usize;

        for _input in self.inputs.iter() {
            size += 40;
        }

        size += VarInt::from(self.outputs.len()).encoded_size() as usize;
        for output in self.outputs.iter() {
            size += output.size();
        }

        size += 4;
        size
    }
}

//@todo this needs to come from our encoding library. Do that today.
impl FromHex for Transaction {
    type Error = FromHexError;

    fn from_hex<T: AsRef<[u8]>>(hex: T) -> Result<Self, Self::Error> {
        let mut buf = Buffer::from(hex::decode(hex).unwrap());

        Ok(Transaction::decode(&mut buf).unwrap())
    }
}

impl Encodable for Transaction {
    //@todo this is going to be the "non-base size"
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

#[cfg(test)]
mod test {
    use super::*;
    use handshake_types::Amount;

    #[test]
    fn test_tx_hashing() {
        let mut input = Input::new_coinbase("");
        let mut outputs = Vec::new();
        input.sequence = 0;
        let mut inputs = Vec::new();
        inputs.push(input);
        let output = Output::new(
            Amount::from_doos(2_000_000_000),
            "ss1qm7zqc7h820qrxd3f72v9jhvmvgzf69cenz8hkn"
                .parse()
                .unwrap(),
        );

        outputs.push(output);
        // let tx: Transaction = Default::default();
        let tx = Transaction::new(391, inputs, outputs);
        let raw = tx.encode();
        dbg!(hex::encode(&raw));
        dbg!(tx.get_base_size());
        let base = tx.get_base_size();
        let witness = raw.len() - base;
        dbg!(witness);
        dbg!(hex::encode(&raw[..base]));
        dbg!(&tx);
        dbg!(hex::encode(tx.hash()));
        dbg!(hex::encode(tx.witness_hash()));
    }
}
