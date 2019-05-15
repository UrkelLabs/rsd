use hex::encode;

//Our version of Buffer that is implemented in bio - > https://github.com/bcoin-org/bufio
#[derive(Default, Debug)]
pub struct Buffer(Vec<u8>);

impl Buffer {
    pub fn new() -> Self {
        Buffer::default()
    }

    //Write u32 in Little Endian format
    //Possibly return the amount of data written //TODO - see if needed anywhere.
    pub fn write_u32(&mut self, data: u32) {
        self.0.extend_from_slice(&data.to_le_bytes());
    }

    pub fn as_hex(&self) -> String {
        encode(&self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::Buffer;

    #[test]
    fn test_write_u32() {
        //u32
        let version: u32 = 123456789;

        let mut buffer = Buffer::new();

        buffer.write_u32(version);

        dbg!(&buffer);

        let hex = buffer.as_hex();

        dbg!(hex);

        // assert_eq!(buffer, [0x65, 0x00, 0x00, 0x00])
    }

}
