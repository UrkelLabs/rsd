use crate::Outpoint;
use extended_primitives::Buffer;
use handshake_encoding::{Decodable, DecodingError, Encodable};
use handshake_script::Witness;
use rand::{thread_rng, Rng};

#[derive(Clone, PartialEq, Debug)]
pub struct Input {
    pub prevout: Outpoint,
    pub sequence: u32,
    pub witness: Witness,
}

impl Input {
    pub fn new_coinbase(flags: &str) -> Input {
        let prevout = Outpoint::null(); //@todo check
        let mut witness = Witness::new();

        let sequence = thread_rng().next_u32();
        let mut random_bytes = [0_u8; 8];
        thread_rng().fill(&random_bytes);

        witness.push_data(Buffer::from(flags));
        witness.push_data(Buffer::from(&random_bytes));
        witness.push_data(Buffer::from(&[0, 8])); //@question -> Ask JJ if this is necessary.

        Input {
            sequence,
            witness: None,
            prevout,
        }
    }
}

impl Encodable for Input {
    fn size(&self) -> usize {
        //prevout (36) + sequence (4)
        40
    }

    fn encode(&self) -> Buffer {
        let mut buffer = Buffer::new();

        buffer.extend(self.prevout.encode());
        buffer.write_u32(self.sequence);

        buffer
    }
}

impl Decodable for Input {
    type Error = DecodingError;

    fn decode(buffer: &mut Buffer) -> Result<Self, Self::Error> {
        let prevout = Outpoint::decode(buffer)?;
        let sequence = buffer.read_u32()?;

        Ok(Input {
            prevout,
            sequence,
            witness: None,
        })
    }
}

//@todo Debug
//@todo Display
//@todo Defaults
//@todo From<TX>
