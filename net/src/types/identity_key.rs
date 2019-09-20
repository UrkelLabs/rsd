use crate::error;
use std::convert::TryFrom;
use std::fmt;
use std::ops;
use std::str::FromStr;

//TODO need a private identity key struct.

#[derive(Clone, Copy)]
pub struct IdentityKey([u8; 33]);

impl IdentityKey {
    pub fn as_array(self) -> [u8; 33] {
        self.0
    }
}

//To hex
//To Base32 }

impl From<[u8; 33]> for IdentityKey {
    fn from(key: [u8; 33]) -> Self {
        IdentityKey(key)
    }
}

impl TryFrom<Vec<u8>> for IdentityKey {
    type Error = error::Error;

    fn try_from(vec: Vec<u8>) -> Result<Self, Self::Error> {
        if vec.len() != 33 {
            return Err(error::Error::InvalidIdentityKey);
        }

        let mut arr = [0; 33];
        arr.copy_from_slice(&vec);

        Ok(IdentityKey(arr))
    }
}

impl ops::Deref for IdentityKey {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

//Remove this when const generics exist.
//TODO review this impl, not sure if it's correct, but essentially PartialEq is impl on slices,
//so we just convert the internal array to a slice and then compare.
impl PartialEq for IdentityKey {
    fn eq(&self, other: &Self) -> bool {
        self.0.as_ref() == other.0.as_ref()
    }
}

impl AsRef<[u8]> for IdentityKey {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl AsMut<[u8]> for IdentityKey {
    fn as_mut(&mut self) -> &mut [u8] {
        &mut self.0
    }
}

impl FromStr for IdentityKey {
    type Err = error::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.len() {
            64 => {
                let mut bytes = [0; 33];
                let bytes_slice = hex::decode(s)?;

                bytes.copy_from_slice(&bytes_slice);

                Ok(IdentityKey(bytes))
            }
            53 => {
                let key = base32::decode(base32::Alphabet::RFC4648 { padding: false }, s)
                    .ok_or_else(|| error::Error::Base32)?;

                let mut bytes = [0; 33];
                bytes.copy_from_slice(&key);

                Ok(IdentityKey(bytes))
            }
            _ => Err(error::Error::InvalidIdentityKey),
        }
    }
}

impl fmt::Debug for IdentityKey {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "IdentityKey: {}", hex::encode(self.0.to_vec()))
    }
}
