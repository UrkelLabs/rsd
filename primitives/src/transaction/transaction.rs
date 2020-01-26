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

        dbg!("error");
        let input_count = buffer.read_varint()?;
        for _ in 0..input_count.as_u64() {
            inputs.push(Input::decode(buffer)?);
        }

        dbg!("error");
        let output_count = buffer.read_varint()?;
        for _ in 0..output_count.as_u64() {
            outputs.push(Output::decode(buffer)?);
        }

        dbg!("error");
        let locktime = buffer.read_u32()?;

        dbg!("error");
        //TODO varint as usize please.
        for i in 0..input_count.as_u64() {
            let witness = Witness::decode(buffer)?;
            inputs[i as usize].witness = witness;
        }
        dbg!("error");

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

    // #[test]
    // fn test_tx_hashing() {
    //     // let mut input = Input::new_coinbase("");
    //     // let mut outputs = Vec::new();
    //     // input.sequence = 0;
    //     // let mut inputs = Vec::new();
    //     // inputs.push(input);
    //     // let output = Output::new(
    //     //     Amount::from_doos(2_000_000_000),
    //     //     "ss1qm7zqc7h820qrxd3f72v9jhvmvgzf69cenz8hkn"
    //     //         .parse()
    //     //         .unwrap(),
    //     // );

    //     // outputs.push(output);
    //     // // let tx: Transaction = Default::default();
    //     // let tx = Transaction::new(391, inputs, outputs);
    //     // let raw = tx.encode();
    //     // dbg!(hex::encode(&raw));
    //     // dbg!(tx.get_base_size());
    //     // let base = tx.get_base_size();
    //     // let witness = raw.len() - base;
    //     // dbg!(witness);
    //     // dbg!(hex::encode(&raw[..base]));
    //     // dbg!(&tx);
    //     // dbg!(hex::encode(tx.hash()));
    //     // dbg!(hex::encode(tx.witness_hash()));
    // }

    #[test]
    fn test_tx_decode() {
        let tx = Transaction::from_hex("0000000002beac67aa84cb3f0354cb018918cfbcfd1b1fb9a693676fa9a4d012a8f673075200000000ffffffff387d93945b2f49295efa3a5e9db435914f03611322dbca0e7eb2f7dee21f2cd400000000ffffffff0278e00100000000000014d320b66463172d77d19aa8296024d264d0957f7507032048e55ada7bca0c8500b78cc53fdf6604d3de4499eb5467dec8d83b2525ad4e83047d000000090000a8000134476508e8a13577000000000014375ebf44978effb90126c1772ac8a160a000f90c0000000000000241d26c53d7a64b45a06e933a3b50d4845b7c6244dc0029634a5465fc45597d12473b5c88b9afcd4d9c7ffc438de582f6404e96e57cde44812378674a21435a2d170121028ea8b984f6be5169edc9e120301883d2bc3d4c8cb8f216fffe88be146c2a3915024148958296317d3e26043e6a190cb46a0f71c4ab0a6a18d357e554c0f2ee91581a1a5362c4527274c7ab15bb5d87f2ff4f4f4e3e37a5010e9a331dc54f386d51050121026321536c96f094474b46f0b27ed1b7403e68a7d3ea83553dd67f85c72394bc68").unwrap();

        // let tx = Transaction::decode(&mut Buffer::from_hex("0000000001559bbf1393486a748bcf06a7c24c8e6ccc6ff4ae595a8b378597b3db6abfef2b00000000ffffffff02000000000000000000144d60300b0d522c835b6568287042b417b40e8343020320cbeeeb6a07c1e8f1fbfbee0ea496b19c0287e8192b4563521dc844bbc7ea1e29040000000003656e64789435770000000000148eba4845a159f30afe580c685d89f69051db26290000000000000241b2e22bff6a35660b429dd952c1b5e3a9209406c8d12ba670c95f87238f2ce5777042e2af5a7f6db3abafb70795ceddc88a1d0a76c60958319174b98328e371ab0121026321536c96f094474b46f0b27ed1b7403e68a7d3ea83553dd67f85c72394bc68").unwrap()).unwrap();

        // let tx = Transaction::decode(&mut Buffer::from_hex("00000000019d732c07a1eccb59e47fda89443d22a6945c9b2f30bd780e10f9fafcc8ab08b200000000ffffffff0200000000000000000014cdf2b6bf02b3b92b37a9bcdea79ee78012f712e80203209a832b1fda4272e8dd5fb8a0248422374551f04baff41c7ab1800abb802764bb04000000000762656c69657665bc9635770000000000148430316e0a59bca8865d508488eecceb9db0dde10000000000000241b0623e757113afa91ba39db44f7503cd71d75a84164a236bfd9c211f23591b224910d8b39ef27a4b5fee16b129a4600947bdd78c2fbb4b53bc524efafd5312380121036145638ba0a602a69147c47e4096df3c6a453847b851b79fc65ec240744494d4").unwrap()).unwrap();
    }
}
