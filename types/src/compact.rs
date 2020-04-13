use extended_primitives::Uint256;
use std::fmt;

#[cfg(feature = "json")]
use encodings::{FromHex, ToHex};
#[cfg(feature = "json")]
use serde::de::{self, Deserialize, Deserializer, MapAccess, SeqAccess, Visitor};

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Compact(u32);

impl From<u32> for Compact {
    fn from(u: u32) -> Self {
        Compact(u)
    }
}

impl From<Compact> for u32 {
    fn from(c: Compact) -> Self {
        c.0
    }
}

impl From<Uint256> for Compact {
    fn from(u: Uint256) -> Self {
        Compact::from_u256(u)
    }
}

impl From<Compact> for Uint256 {
    fn from(c: Compact) -> Self {
        // ignore overflows and negative values
        c.to_u256().unwrap_or_else(|x| x)
    }
}

//@todo to_target -> Returns a hash.
//@todo from_target -> takes in a hash
//@todo to_difficulty -> Returns f64
//@todo from_difficulty -> from f64
impl Compact {
    pub fn new(u: u32) -> Self {
        Compact(u)
    }

    pub fn max_value() -> Self {
        Uint256::max_value().into()
    }

    /// Computes the target [0, T] that a blockhash must land in to be valid
    /// Returns value in error, if there is an overflow or its negative value
    pub fn to_u256(&self) -> Result<Uint256, Uint256> {
        if self.0 == 0 {
            return Ok(Uint256::from_u64(0).unwrap());
        }
        let exponent = self.0 >> 24;
        let negative = (self.0 >> 23) & 1;

        let mut mantissa = self.0 & 0x_7ff_fff;

        let result = if exponent <= 3 {
            mantissa >>= 8 * (3 - exponent as usize);
            Uint256::from(mantissa)
        } else {
            Uint256::from(mantissa) << (8 * (exponent as usize - 3))
        };

        let overflow = (mantissa != 0 && exponent > 34)
            || (mantissa > 0xff && exponent > 33)
            || (mantissa > 0xffff && exponent > 32);

        if negative != 0 || overflow {
            Err(result)
        } else {
            Ok(result)
        }
    }

    pub fn from_u256(val: Uint256) -> Self {
        let mut size = (val.bits() + 7) / 8;
        let mut compact = if size <= 3 {
            (val.low_u64() << (8 * (3 - size))) as u32
        } else {
            let bn = val >> (8 * (size - 3));
            bn.low_u32()
        };

        if (compact & 0x00800000) != 0 {
            compact >>= 8;
            size += 1;
        }

        assert!((compact & !0x_7ff_fff) == 0);
        assert!(size < 256);
        Compact(compact | (size << 24) as u32)
    }

    pub fn to_f64(&self) -> f64 {
        let mut shift = (self.0 >> 24) & 0xff;
        let mut diff = f64::from(0x0000ffffu32) / f64::from(self.0 & 0x00ffffffu32);
        while shift < 29 {
            diff *= f64::from(256);
            shift += 1;
        }
        while shift > 29 {
            diff /= f64::from(256.0);
            shift -= 1;
        }
        diff
    }
}

// ====== Feature: JSON ======

#[cfg(feature = "json")]
impl serde::Serialize for Compact {
    fn serialize<S: serde::Serializer>(&self, s: S) -> std::result::Result<S::Ok, S::Error> {
        s.serialize_str(&self.0.to_be_bytes().to_hex())
    }
}

#[cfg(feature = "json")]
impl<'de> Deserialize<'de> for Compact {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct CompactVisitor;

        impl CompactVisitor {
            pub fn new() -> CompactVisitor {
                CompactVisitor {}
            }
        }

        impl<'de> Visitor<'de> for CompactVisitor {
            type Value = Compact;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("compact")
            }

            fn visit_u32<E>(self, value: u32) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(Compact(value))
            }

            fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                // Ok(i32::from(value))
                if value >= u64::from(u32::min_value()) && value <= u64::from(u32::max_value()) {
                    Ok(Compact(value as u32))
                } else {
                    Err(E::custom(format!("u32 out of range: {}", value)))
                }
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                let bytes = match Vec::from_hex(value) {
                    Ok(bytes) => bytes,
                    Err(e) => return Err(E::custom(e)),
                };

                if bytes.len() == 4 {
                    let mut bytes_array = [0; 4];
                    bytes_array.copy_from_slice(&bytes);
                    let val = u32::from_be_bytes(bytes_array);
                    Ok(Compact(val))
                } else {
                    Err(E::custom("Compact out of range"))
                }
            }
        }

        deserializer.deserialize_any(CompactVisitor::new())
    }
}

#[cfg(test)]
mod tests {
    use super::Compact;
    use super::*;

    #[test]
    fn test_compact_to_u256() {
        assert_eq!(Compact::new(0x01003456).to_u256(), Ok(0u64.into()));
        assert_eq!(Compact::new(0x01123456).to_u256(), Ok(0x12u64.into()));
        assert_eq!(Compact::new(0x02008000).to_u256(), Ok(0x80u64.into()));
        assert_eq!(Compact::new(0x05009234).to_u256(), Ok(0x92340000u64.into()));
        // negative -0x12345600
        assert!(Compact::new(0x04923456).to_u256().is_err());
        assert_eq!(Compact::new(0x04123456).to_u256(), Ok(0x12345600u64.into()));
    }

    #[test]
    fn test_from_u256() {
        let test1 = Uint256::from(1000u64);
        assert_eq!(Compact::new(0x0203e800), Compact::from_u256(test1));

        // let test2 = Uint256::from(2).pow(Uint256::from(256 - 32)) - Uint256::from(1);
        // assert_eq!(Compact::new(0x1d00ffff), Compact::from_u256(test2));
    }

    #[test]
    fn test_compact_to_from_u256() {
        // TODO: it does not work both ways for small values... check why
        let compact = Compact::new(0x1d00ffff);
        let compact2 = Compact::from_u256(compact.to_u256().unwrap());
        assert_eq!(compact, compact2);

        let compact = Compact::new(0x05009234);
        let compact2 = Compact::from_u256(compact.to_u256().unwrap());
        assert_eq!(compact, compact2);
    }

    #[test]
    fn difficulty() {
        fn compare_f64(v1: f64, v2: f64) -> bool {
            (v1 - v2).abs() < 0.00001
        }

        assert!(compare_f64(Compact::new(0x1b0404cb).to_f64(), 16307.42094));

        // tests from original bitcoin client:
        // https://github.com/bitcoin/bitcoin/blob/1e8f88e071019907785b260477bd359bef6f9a8f/src/test/blockchain_tests.cpp

        assert!(compare_f64(Compact::new(0x1f111111).to_f64(), 0.000001));
        assert!(compare_f64(Compact::new(0x1ef88f6f).to_f64(), 0.000016));
        assert!(compare_f64(Compact::new(0x1df88f6f).to_f64(), 0.004023));
        assert!(compare_f64(Compact::new(0x1cf88f6f).to_f64(), 1.029916));
        assert!(compare_f64(
            Compact::new(0x12345678).to_f64(),
            5913134931067755359633408.0
        ));
    }
}
