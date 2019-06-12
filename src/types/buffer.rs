use crate::types::{Hash, Uint256};
use hex::encode;
use std::ops;

//Our version of Buffer that is implemented in bio - > https://github.com/bcoin-org/bufio
#[derive(Default, Debug, PartialEq)]
pub struct Buffer(Vec<u8>);

impl Buffer {
    pub fn new() -> Self {
        Buffer::default()
    }

    //Unsigned Integers - Little Endian
    pub fn write_u8(&mut self, data: u8) {
        self.0.extend_from_slice(&data.to_le_bytes());
    }

    pub fn write_u16(&mut self, data: u16) {
        self.0.extend_from_slice(&data.to_le_bytes());
    }

    pub fn write_u32(&mut self, data: u32) {
        self.0.extend_from_slice(&data.to_le_bytes());
    }

    pub fn write_u64(&mut self, data: u64) {
        self.0.extend_from_slice(&data.to_le_bytes());
    }

    //TODO u128

    pub fn write_u256(&mut self, data: Uint256) {
        self.0.extend_from_slice(&data.to_le_bytes());
    }

    //Big Endian
    pub fn write_u8_be(&mut self, data: u8) {
        self.0.extend_from_slice(&data.to_be_bytes());
    }

    pub fn write_u16_be(&mut self, data: u16) {
        self.0.extend_from_slice(&data.to_be_bytes());
    }

    pub fn write_u32_be(&mut self, data: u32) {
        self.0.extend_from_slice(&data.to_be_bytes());
    }

    pub fn write_u64_be(&mut self, data: u64) {
        self.0.extend_from_slice(&data.to_be_bytes());
    }

    //TODO u128, and u256

    //Signed Integers
    pub fn write_i8(&mut self, data: u8) {
        self.0.extend_from_slice(&data.to_le_bytes());
    }

    pub fn write_i16(&mut self, data: u16) {
        self.0.extend_from_slice(&data.to_le_bytes());
    }

    pub fn write_i32(&mut self, data: u32) {
        self.0.extend_from_slice(&data.to_le_bytes());
    }

    pub fn write_i64(&mut self, data: u64) {
        self.0.extend_from_slice(&data.to_le_bytes());
    }

    //Big Endian
    pub fn write_i8_be(&mut self, data: u8) {
        self.0.extend_from_slice(&data.to_be_bytes());
    }

    pub fn write_i16_be(&mut self, data: u16) {
        self.0.extend_from_slice(&data.to_be_bytes());
    }

    pub fn write_i32_be(&mut self, data: u32) {
        self.0.extend_from_slice(&data.to_be_bytes());
    }

    pub fn write_i64_be(&mut self, data: u64) {
        self.0.extend_from_slice(&data.to_be_bytes());
    }

    pub fn write_bytes(&mut self, bytes: &[u8]) {
        //TODO should we clone here or just pass in
        self.0.extend_from_slice(bytes);
    }

    pub fn write_str(&mut self, string: &str) {
        self.0.extend_from_slice(string.as_bytes());
    }

    pub fn write_string(&mut self, string: String) {
        self.0.extend_from_slice(string.as_bytes());
    }

    pub fn write_hash(&mut self, hash: Hash) {
        self.0.extend(&hash.to_array());
    }

    pub fn fill(&mut self, value: u8, amount: usize) {
        let fill_amount = vec![value; amount];
        self.0.extend(fill_amount);
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

impl From<Vec<u8>> for Buffer {
    fn from(buf: Vec<u8>) -> Self {
        Buffer(buf)
    }
}

//TODO review, seems inefficent
impl From<&str> for Buffer {
    fn from(buf: &str) -> Self {
        Buffer(buf.as_bytes().to_vec())
    }
}

//TODO review, seems inefficent
impl From<String> for Buffer {
    fn from(buf: String) -> Self {
        Buffer(buf.as_bytes().to_vec())
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
