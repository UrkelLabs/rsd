#[derive(Copy, Clone, PartialEq, Eq, Default, PartialOrd, Ord, Hash, Debug)]
pub struct Hash([u8; 32]);

impl Hash {
    pub fn to_array(&self) -> [u8; 32] {
        self.0
    }

    pub fn to_string(&self) -> String {
        hex::encode(self.0)
    }
}

//
//We can have it from string, or we can have it be from hex //TODO both might be useful.
//Need more checks here for length, and errors
impl From<String> for Hash {
    fn from(hex: String) -> Self {
        //Do not unwrap here, we need to catch this error.
        let raw = hex::decode(hex).unwrap();
        // let hash: &[32] = &raw;
        // Hash(raw.try_into())
        Hash::from(raw)
    }
}

impl From<&str> for Hash {
    fn from(hex: &str) -> Self {
        //Do not unwrap here, we need to catch this error.
        let raw = hex::decode(hex).unwrap();
        // let hash: &[32] = &raw;
        // Hash(raw.try_into())
        Hash::from(raw)
    }
}

//Need more checks here for length, and errors
impl From<Vec<u8>> for Hash {
    fn from(hex_vec: Vec<u8>) -> Self {
        let mut array = [0; 32];
        array.copy_from_slice(&hex_vec);
        Hash(array)
    }
}

//This should only be implemented on Blake2b hash
//Redo this when we split to blake2b/ run into problems TODO
impl From<[u8; 32]> for Hash {
    fn from(bytes: [u8; 32]) -> Self {
        Hash(bytes)
    }
}
