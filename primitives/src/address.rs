use bech32::{u5, FromBase32, ToBase32};
use extended_primitives::Buffer;
use handshake_encoding::{Decodable, DecodingError, Encodable};
use std::fmt;
use std::str::FromStr;

#[cfg(feature = "json")]
use encodings::ToHex;
#[cfg(feature = "json")]
use serde::de::{self, Deserialize, Deserializer, MapAccess, SeqAccess, Visitor};
#[cfg(feature = "json")]
use serde::ser::SerializeStruct;

//@todo we need a toHS1 syntax function.
//bech32

#[derive(Debug)]
pub enum AddressError {
    InvalidAddressVersion,
    InvalidAddressSize,
    InvalidNetworkPrefix,
    InvalidHash,
    Bech32(bech32::Error),
}

impl From<bech32::Error> for AddressError {
    fn from(e: bech32::Error) -> Self {
        AddressError::Bech32(e)
    }
}

impl From<AddressError> for DecodingError {
    fn from(e: AddressError) -> DecodingError {
        DecodingError::InvalidData(format!("{:?}", e))
    }
}

impl fmt::Display for AddressError {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match self {
            _ => formatter.write_str("todo"),
        }
    }
}

// #[derive(PartialEq, Clone, Debug, Copy)]
#[derive(PartialEq, Clone, Debug)]
pub enum Payload {
    PubkeyHash(Buffer),
    ScriptHash(Buffer),
    Unknown(Buffer),
}

impl Payload {
    pub fn len(&self) -> usize {
        match self {
            Payload::PubkeyHash(hash) => hash.len(),
            Payload::ScriptHash(hash) => hash.len(),
            Payload::Unknown(hash) => hash.len(),
        }
    }

    pub fn is_empty(&self) -> bool {
        match self {
            Payload::PubkeyHash(hash) => hash.is_empty(),
            Payload::ScriptHash(hash) => hash.is_empty(),
            Payload::Unknown(hash) => hash.is_empty(),
        }
    }

    pub fn to_hash(self) -> Buffer {
        match self {
            Payload::PubkeyHash(hash) => hash,
            Payload::ScriptHash(hash) => hash,
            Payload::Unknown(hash) => hash,
        }
    }

    pub fn as_hash(&self) -> &Buffer {
        match self {
            Payload::PubkeyHash(hash) => hash,
            Payload::ScriptHash(hash) => hash,
            Payload::Unknown(hash) => hash,
        }
    }

    pub fn from_hash(hash: Buffer) -> Result<Payload, AddressError> {
        match hash.len() {
            20 => Ok(Payload::PubkeyHash(hash)),
            32 => Ok(Payload::ScriptHash(hash)),
            _ => Ok(Payload::Unknown(hash)),
        }
    }
}

//@todo Impl FromHex, ToHex

//@todo ideally implement copy here, but we need to implement it for Buffer, and we really need to
//look into performance degration there.
// #[derive(PartialEq, Clone, Debug, Copy)]
#[derive(PartialEq, Clone, Debug)]
pub struct Address {
    //Can we make this u8? TODO
    //And do we even need this?
    pub version: u8,
    pub hash: Payload,
}

impl Address {
    pub fn new(version: u8, hash: Payload) -> Self {
        Address { version, hash }
    }

    //TODO
    // pub fn is_null(&self) -> bool {
    //     self.hash.is_null()
    // }

    pub fn is_null_data(&self) -> bool {
        self.version == 31
    }

    pub fn is_unspendable(&self) -> bool {
        self.is_null_data()
    }

    pub fn to_bech32(&self) -> String {
        //Also todo this should probably just be in toString, and should use writers so that we
        //don't allocate.
        //@todo this should be network dependant. Need to put work into this.
        //Right now this will only support mainnet addresses.
        // let mut data = vec![self.version];
        let mut data = vec![bech32::u5::try_from_u8(self.version).unwrap()];
        data.extend_from_slice(&self.hash.clone().to_hash().to_base32());

        bech32::encode("hs", data).unwrap()
    }
}

//@todo review if this is a good default. Should be triggered on "null"
impl Default for Address {
    fn default() -> Self {
        Address {
            version: 0,
            hash: Payload::PubkeyHash(Buffer::new()),
        }
    }
}

impl Decodable for Address {
    type Err = DecodingError;

    fn decode(buffer: &mut Buffer) -> Result<Self, Self::Err> {
        let version = buffer.read_u8()?;

        if version > 31 {
            return Err(DecodingError::InvalidData(
                "Invalid Address Version".to_string(),
            ));
        }

        let size = buffer.read_u8()?;

        if size < 2 || size > 40 {
            return Err(DecodingError::InvalidData(
                "Invalid Address Size".to_string(),
            ));
        }

        let hash = buffer.read_bytes(size as usize)?;

        let hash = Payload::from_hash(Buffer::from(hash))?;

        Ok(Address { version, hash })
    }
}

impl Encodable for Address {
    fn size(&self) -> usize {
        1 + 1 + self.hash.len()
    }

    fn encode(&self) -> Buffer {
        let mut buffer = Buffer::new();

        buffer.write_u8(self.version);
        buffer.write_u8(self.hash.len() as u8);
        //TODO fix this
        buffer.extend(self.hash.as_hash().clone());

        buffer
    }
}

impl FromStr for Address {
    type Err = AddressError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        //@todo should we be checking network here?
        let (_hrp, data) = bech32::decode(s)?;

        let (version, hash) = version_hash_from_bech32(data);

        let hash = Payload::from_hash(hash)?;

        Ok(Address { version, hash })
    }
}

// //TODO eq, partial eq, ordering.

fn version_hash_from_bech32(data: Vec<u5>) -> (u8, Buffer) {
    let (version, d) = data.split_at(1);

    let hash_data = Vec::from_base32(d).unwrap();

    let mut hash = Buffer::new();

    for elem in hash_data.iter() {
        hash.write_u8(*elem);
    }

    (version[0].to_u8(), hash)
}

#[cfg(feature = "json")]
impl serde::Serialize for Address {
    fn serialize<S: serde::Serializer>(&self, s: S) -> std::result::Result<S::Ok, S::Error> {
        let mut state = s.serialize_struct("Address", 2)?;
        state.serialize_field("version", &self.version)?;
        state.serialize_field("hash", &self.hash.as_hash().to_hex())?;
        state.end()
    }
}

#[cfg(feature = "json")]
impl<'de> Deserialize<'de> for Address {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        enum Field {
            Version,
            Hash,
            Str,
        };

        impl<'de> Deserialize<'de> for Field {
            fn deserialize<D>(deserializer: D) -> Result<Field, D::Error>
            where
                D: Deserializer<'de>,
            {
                struct FieldVisitor;

                impl<'de> Visitor<'de> for FieldVisitor {
                    type Value = Field;

                    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                        formatter.write_str("`version` or `hash`")
                    }

                    fn visit_str<E>(self, value: &str) -> Result<Field, E>
                    where
                        E: de::Error,
                    {
                        match value {
                            "version" => Ok(Field::Version),
                            "hash" => Ok(Field::Hash),
                            "string" => Ok(Field::Str),
                            _ => Err(de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }

                deserializer.deserialize_identifier(FieldVisitor)
            }
        }

        struct AddressVisitor;

        impl<'de> Visitor<'de> for AddressVisitor {
            type Value = Address;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct Address")
            }

            fn visit_seq<V>(self, mut seq: V) -> Result<Address, V::Error>
            where
                V: SeqAccess<'de>,
            {
                //Skip string
                seq.next_element()?
                    .ok_or_else(|| de::Error::invalid_length(0, &self))?;
                let version = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(1, &self))?;
                let hash_raw: Buffer = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(2, &self))?;

                let hash = Payload::from_hash(hash_raw).map_err(de::Error::custom)?;

                Ok(Address::new(version, hash))
            }

            fn visit_str<E>(self, value: &str) -> Result<Address, E>
            where
                E: de::Error,
            {
                Ok(Address::from_str(value).map_err(de::Error::custom)?)
            }

            fn visit_map<V>(self, mut map: V) -> Result<Address, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut version = None;
                let mut hash = None;
                while let Some(key) = map.next_key()? {
                    match key {
                        Field::Version => {
                            if version.is_some() {
                                return Err(de::Error::duplicate_field("version"));
                            }
                            version = Some(map.next_value()?);
                        }
                        Field::Hash => {
                            if hash.is_some() {
                                return Err(de::Error::duplicate_field("hash"));
                            }
                            hash = Some(map.next_value()?);
                        }
                        Field::Str => {}
                    }
                }
                let version = version.ok_or_else(|| de::Error::missing_field("version"))?;
                let hash_raw = hash.ok_or_else(|| de::Error::missing_field("hash"))?;

                let hash = Payload::from_hash(hash_raw).map_err(de::Error::custom)?;
                Ok(Address::new(version, hash))
            }
        }

        const FIELDS: &'static [&'static str] = &["version", "hash"];
        deserializer.deserialize_struct("Address", FIELDS, AddressVisitor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_from_bech32() {
        let addr = Address::from_str("hs1qd42hrldu5yqee58se4uj6xctm7nk28r70e84vx").unwrap();
        dbg!(&addr);

        dbg!(addr.to_bech32());
    }

    #[test]
    fn test_from_unknown() {
        let addr = Address::from_str("hs1lqqqqhuxwgy");

        dbg!(addr);
    }
}
