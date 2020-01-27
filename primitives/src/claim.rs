use encodings::hex::{FromHex, ToHex};
use extended_primitives::Buffer;
use handshake_encoding::{Decodable, DecodingError, Encodable};

pub struct Claim {
    pub blob: Buffer,
}

impl Claim {
    pub fn new(blob: Buffer) -> Self {
        Claim { blob }
    }
}

impl Encodable for Claim {
    fn size(&self) -> usize {
        2 + self.blob.len()
    }

    fn encode(&self) -> Buffer {
        let mut buffer = Buffer::new();

        buffer.write_u16(self.blob.len() as u16);
        buffer.write_bytes(&self.blob);

        buffer
    }
}

impl Decodable for Claim {
    type Err = DecodingError;

    fn decode(buffer: &mut Buffer) -> Result<Self, Self::Err> {
        let size = buffer.read_u16()?;

        //@todo
        // if (data.length > 2 + 10000)
        // throw new Error('Proof too large.');
        //  if (size > 10000)
        // throw new Error('Invalid claim size.');

        let blob = Buffer::from(buffer.read_bytes(size as usize)?);

        //@todo
        // if (br.left() !== 0)
        // throw new Error('Trailing data.');

        Ok(Claim { blob })
    }
}

impl ToHex for Claim {
    fn to_hex(&self) -> String {
        self.encode().to_hex()
    }
}

//@todo not sure if I like this function it assumes that the claim is encoded w/ it's size - which
//I'm not sure we will see all that often. This really more depends on if we see this in network
//packets. Just be aware that this will not work for block templte ClaimEntries
impl FromHex for Claim {
    type Error = DecodingError;

    fn from_hex<T: AsRef<[u8]>>(hex: T) -> std::result::Result<Self, Self::Error> {
        Claim::decode(&mut Buffer::from_hex(hex)?)
    }
}
