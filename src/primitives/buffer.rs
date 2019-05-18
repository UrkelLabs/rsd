use crate::primitives::hash::Hash;
use hex::encode;
use std::ops;

//Our version of Buffer that is implemented in bio - > https://github.com/bcoin-org/bufio
#[derive(Default, Debug, PartialEq)]
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

    pub fn write_u64(&mut self, data: u64) {
        self.0.extend_from_slice(&data.to_le_bytes());
    }

    pub fn write_hash(&mut self, hash: Hash) {
        self.0.extend(&hash.to_array());
    }

    //Return Hex string of the buffer.
    pub fn to_hex(&self) -> String {
        encode(&self.0)
    }

    //Return Hex string of the buffer, Consumes the Hex
    pub fn into_hex(self) -> String {
        encode(self.0)
    }
}

//Allows us to grab specific bytes from the buffer e.g.
//grab the merkle tree from the middle of the buffer.
impl ops::Deref for Buffer {
    type Target = Vec<u8>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

//Allows us to grab specific bytes from the buffer e.g.
//grab the merkle tree from the middle of the buffer.
//Same as above, but allows us to grab those bytes and mutable, thus changing them without
//having to allocate more mem.
impl ops::DerefMut for Buffer {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

//Allows Buffer to be used as a reference for a [u8] TODO double check this.
//And thoroughly comment for everyone
impl AsRef<[u8]> for Buffer {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

//Allows Buffer to be used as a mut for a [u8] TODO double check this.
//And thoroughly comment for everyone
impl AsMut<[u8]> for Buffer {
    fn as_mut(&mut self) -> &mut [u8] {
        &mut self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_write_u32() {
        let version: u32 = 123456789;

        let mut buffer = Buffer::new();

        buffer.write_u32(version);

        assert_eq!(buffer, Buffer([21, 205, 91, 7].to_vec()));
    }

    #[test]
    fn test_write_hash() {
        let hash = Hash::from("bb42edce1895f9a969e81d7371ec113a0966e5d55035a84f87ca098e4f0a1a86");

        let mut buffer = Buffer::new();

        buffer.write_hash(hash);

        dbg!(buffer);
    }

    #[test]
    fn test_to_hex() {
        let version: u32 = 123456789;

        let mut buffer = Buffer::new();

        buffer.write_u32(version);

        assert_eq!(buffer, Buffer([21, 205, 91, 7].to_vec()));

        let hex = buffer.to_hex();

        assert_eq!(hex, "15cd5b07")
    }

    #[test]
    fn test_into_hex() {
        let version: u32 = 123456789;

        let mut buffer = Buffer::new();

        buffer.write_u32(version);

        assert_eq!(buffer, Buffer([21, 205, 91, 7].to_vec()));

        let hex = buffer.into_hex();

        assert_eq!(hex, "15cd5b07")
    }

}
