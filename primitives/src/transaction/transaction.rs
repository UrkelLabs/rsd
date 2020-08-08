use crate::{Input, Output};
use cryptoxide::blake2b::Blake2b;
use cryptoxide::digest::Digest;
use encodings::hex::{FromHex, FromHexError, ToHex};
use extended_primitives::{Buffer, Hash, VarInt};
use handshake_encoding::{Decodable, DecodingError, Encodable};
use handshake_script::Witness;

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

    pub fn get_witness_size(&self) -> usize {
        let mut size = 0;

        for input in &self.inputs {
            size += input.witness.var_size();
        }

        size
    }
}

//@todo - thought. This could be automatically derived from Decoding/Encoding. Let's test this out
//later today, and just put it into the encoding library.
impl FromHex for Transaction {
    type Error = DecodingError;

    fn from_hex<T: AsRef<[u8]>>(hex: T) -> Result<Self, Self::Error> {
        Transaction::decode(&mut Buffer::from_hex(hex)?)
    }
}

impl ToHex for Transaction {
    fn to_hex(&self) -> String {
        self.encode().to_hex()
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
    fn test_tx_decode() {
        let hex = "0000000001559bbf1393486a748bcf06a7c24c8e6ccc6ff4ae595a8b378597b3db6abfef2b00000000ffffffff02000000000000000000144d60300b0d522c835b6568287042b417b40e8343020320cbeeeb6a07c1e8f1fbfbee0ea496b19c0287e8192b4563521dc844bbc7ea1e29040000000003656e64789435770000000000148eba4845a159f30afe580c685d89f69051db26290000000000000241b2e22bff6a35660b429dd952c1b5e3a9209406c8d12ba670c95f87238f2ce5777042e2af5a7f6db3abafb70795ceddc88a1d0a76c60958319174b98328e371ab0121026321536c96f094474b46f0b27ed1b7403e68a7d3ea83553dd67f85c72394bc68";
        let tx = Transaction::from_hex(hex).unwrap();
        let encoded = tx.to_hex();

        assert_eq!(hex, encoded);
    }

    #[test]
    fn test_tx_hash() {
        let hex = "000000000189b8fefcbc040b03b3de29e1d7836716379e0282dfcd8d8be693431e15ba1bc301000000ffffffff0200000000000000000014f6448adf6a19ef63cbf7a00404b6b3c4bad059960203208323ce31d4f62e2c2a89e1aaffd4ed31cef82b3a462bfd4a9007b1a78b9ae49b04000000000b63616e6e61627574746572e7203193000000000014fce9ecbb20189c047d4c7d57339202066e0123b30000000000000241c5bc2bae9ff6c64c0c128fd76a79522be531a84bb96cc81c2eacb5d8591752b15821bf062e706c19c9d3be8b39eb088ec7a530fc5ae88985074ac1f5ba0c1c7d0121023a78c14e78410e7082bd82aa73d219964ff5d047b18ba80e9912bcfe3e9007f5";
        let tx = Transaction::from_hex(hex).unwrap();

        let hash = tx.hash();
        let expected = Hash::from_hex("baac577630d12c95c49cdf99e53e4e0c5b24f54501bbbc7524c76bb0ae52dac5").unwrap();
        assert_eq!(hash, expected);

        let expected_witness_hash = Hash::from_hex("4426bd7f4ddc952acbee69c533fd988e67e499b94398924a8c46fe7ea59961b9").unwrap();
        let witness_hash = tx.witness_hash();
        assert_eq!(witness_hash, expected_witness_hash);
    }

    #[test]
    fn test_tx_size() {
        let hex = "00000000017aa330f0a00a86b04df4569ef4dfeea0f5462fa3907378ce68b45a1e9ed6f97801000000ffffffff020000000000000000001430d02d1527cd490744111efe18524ebb531224810203205e5bd7f3e454fb4a4c80644ed1dbd66ec3d9ba6e7845704617dd8d325d2aaa890400000000066f63756c61723078fa34000000000014a1ddaf498c30cbae43c031bbfa7871fcde94966b00000000000002413a1d533b29d4f2618f61fffbdfc0b22a489838d0b743346a93b65445e15f93bf653cccd2e89d10a2792e10c0f983ebff04df4d810ec09246cdfdd7383e8bee420121024d80b165d51c32aa30076878279dfb87f4a149fc3fe3fd3d49ec757a621304bf";
        let tx = Transaction::from_hex(hex).unwrap();

        let base_size = tx.get_base_size();
        let witness_size = tx.get_witness_size();

        assert_eq!(base_size, 159);
        assert_eq!(witness_size, 101);
    }
}
