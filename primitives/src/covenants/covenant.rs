use encodings::hex::{FromHex, ToHex};
use extended_primitives::{Buffer, VarInt};
use handshake_encoding::{Decodable, DecodingError, Encodable};

use super::{
    BidCovenant, ClaimCovenant, FinalizeCovenant, OpenCovenant, RedeemCovenant, RegisterCovenant,
    RenewCovenant, RevealCovenant, RevokeCovenant, TransferCovenant, UpdateCovenant,
};

#[cfg(feature = "json")]
use serde::de::{self, Deserialize, Deserializer, MapAccess, SeqAccess, Visitor};
#[cfg(feature = "json")]
use serde::ser::SerializeStruct;

/// A Handshake covenant, which is a method of changing name state on the chain.
#[derive(PartialEq, Clone, Debug)]
pub enum Covenant {
    None,
    Claim(ClaimCovenant),
    Bid(BidCovenant),
    Open(OpenCovenant),
    Reveal(RevealCovenant),
    Redeem(RedeemCovenant),
    Register(RegisterCovenant),
    Update(UpdateCovenant),
    Renew(RenewCovenant),
    Transfer(TransferCovenant),
    Finalize(FinalizeCovenant),
    Revoke(RevokeCovenant),
}

impl Covenant {
    pub fn is_name(&self) -> bool {
        match self {
            Covenant::None => false,
            _ => true,
        }
    }

    pub fn is_dustworthy(&self) -> bool {
        match self {
            Covenant::None => true,
            Covenant::Bid(_) => true,
            _ => false,
        }
    }

    pub fn is_linked(&self) -> bool {
        match self {
            Covenant::None => false,
            Covenant::Claim(_) => false,
            Covenant::Open(_) => false,
            Covenant::Bid(_) => false,
            _ => true,
        }
    }

    //Returns whether the Covenant is spendable or not.
    pub fn is_unspendable(&self) -> bool {
        match self {
            Covenant::Revoke(_) => true,
            _ => false,
        }
    }

    //Returns whether or not the Coin inside of the covenant is spendable.
    pub fn is_nonspendable(&self) -> bool {
        match self {
            Covenant::None => false,
            Covenant::Open(_) => false,
            Covenant::Redeem(_) => false,
            _ => true,
        }
    }

    pub fn get_type(&self) -> u8 {
        match self {
            Covenant::None => 0,
            Covenant::Claim(_) => 1,
            Covenant::Bid(_) => 2,
            Covenant::Open(_) => 3,
            Covenant::Reveal(_) => 4,
            Covenant::Redeem(_) => 5,
            Covenant::Register(_) => 6,
            Covenant::Update(_) => 7,
            Covenant::Renew(_) => 8,
            Covenant::Transfer(_) => 9,
            Covenant::Finalize(_) => 10,
            Covenant::Revoke(_) => 11,
        }
    }

    pub fn get_action(&self) -> String {
        match self {
            Covenant::None => String::from("NONE"),
            Covenant::Claim(_) => String::from("CLAIM"),
            Covenant::Bid(_) => String::from("BID"),
            Covenant::Open(_) => String::from("OPEN"),
            Covenant::Reveal(_) => String::from("REVEAL"),
            Covenant::Redeem(_) => String::from("REDEEM"),
            Covenant::Register(_) => String::from("REGISTER"),
            Covenant::Update(_) => String::from("UPDATE"),
            Covenant::Renew(_) => String::from("RENEW"),
            Covenant::Transfer(_) => String::from("TRANSFER"),
            Covenant::Finalize(_) => String::from("FINALIZE"),
            Covenant::Revoke(_) => String::from("REVOKE"),
        }
    }

    pub fn get_items(&self) -> Vec<Buffer> {
        match self {
            Covenant::None => Vec::new(),
            Covenant::Claim(claim) => claim.get_items(),
            Covenant::Bid(bid) => bid.get_items(),
            Covenant::Open(open) => open.get_items(),
            Covenant::Reveal(reveal) => reveal.get_items(),
            Covenant::Redeem(redeem) => redeem.get_items(),
            Covenant::Register(register) => register.get_items(),
            Covenant::Update(update) => update.get_items(),
            Covenant::Renew(renew) => renew.get_items(),
            Covenant::Transfer(transfer) => transfer.get_items(),
            Covenant::Finalize(finalize) => finalize.get_items(),
            Covenant::Revoke(revoke) => revoke.get_items(),
        }
    }
}

impl Decodable for Covenant {
    type Err = DecodingError;

    fn decode(buffer: &mut Buffer) -> Result<Self, Self::Err> {
        let covenant_type = buffer.read_u8()?;

        match covenant_type {
            //@todo I don't think this will work. Still need to read the varint on this one.
            0 => {
                //Need to do this since it's encoded no matter what.
                buffer.read_varint()?;
                Ok(Covenant::None)
            }
            1 => {
                let covenant = ClaimCovenant::decode(buffer)?;
                Ok(Covenant::Claim(covenant))
            }
            2 => {
                let covenant = OpenCovenant::decode(buffer)?;
                Ok(Covenant::Open(covenant))
            }
            3 => {
                let covenant = BidCovenant::decode(buffer)?;
                Ok(Covenant::Bid(covenant))
            }
            4 => {
                let covenant = RevealCovenant::decode(buffer)?;
                Ok(Covenant::Reveal(covenant))
            }
            5 => {
                let covenant = RedeemCovenant::decode(buffer)?;
                Ok(Covenant::Redeem(covenant))
            }
            6 => {
                let covenant = RegisterCovenant::decode(buffer)?;
                Ok(Covenant::Register(covenant))
            }
            7 => {
                let covenant = UpdateCovenant::decode(buffer)?;
                Ok(Covenant::Update(covenant))
            }
            8 => {
                let covenant = RenewCovenant::decode(buffer)?;
                Ok(Covenant::Renew(covenant))
            }
            9 => {
                let covenant = TransferCovenant::decode(buffer)?;
                Ok(Covenant::Transfer(covenant))
            }
            10 => {
                let covenant = FinalizeCovenant::decode(buffer)?;
                Ok(Covenant::Finalize(covenant))
            }
            11 => {
                let covenant = RevokeCovenant::decode(buffer)?;
                Ok(Covenant::Revoke(covenant))
            }
            _ => Err(DecodingError::InvalidData(
                "Unknown Covenant Type".to_owned(),
            )),
        }
    }
}

impl Encodable for Covenant {
    //@todo going to code this as getVarsize right now, can change later.
    fn size(&self) -> usize {
        let mut size = 1;
        size += match self {
            Covenant::None => VarInt::from(0 as u64).encoded_size() as usize,
            Covenant::Claim(claim) => claim.size(),
            Covenant::Open(open) => open.size(),
            Covenant::Bid(bid) => bid.size(),
            Covenant::Reveal(reveal) => reveal.size(),
            Covenant::Redeem(redeem) => redeem.size(),
            Covenant::Register(register) => register.size(),
            Covenant::Update(update) => update.size(),
            Covenant::Renew(renew) => renew.size(),
            Covenant::Transfer(transfer) => transfer.size(),
            Covenant::Finalize(finalize) => finalize.size(),
            Covenant::Revoke(revoke) => revoke.size(),
        };
        size
    }

    fn encode(&self) -> Buffer {
        match self {
            Covenant::None => {
                let mut buf = Buffer::new();
                buf.write_u8(0);
                buf.write_varint(0);
                buf
            }
            Covenant::Claim(claim) => claim.encode(),
            Covenant::Open(open) => open.encode(),
            Covenant::Bid(bid) => bid.encode(),
            Covenant::Reveal(reveal) => reveal.encode(),
            Covenant::Redeem(redeem) => redeem.encode(),
            Covenant::Register(register) => register.encode(),
            Covenant::Update(update) => update.encode(),
            Covenant::Renew(renew) => renew.encode(),
            Covenant::Transfer(transfer) => transfer.encode(),
            Covenant::Finalize(finalize) => finalize.encode(),
            Covenant::Revoke(revoke) => revoke.encode(),
        }
    }
}

impl ToHex for Covenant {
    fn to_hex(&self) -> String {
        self.encode().to_hex()
    }
}

impl FromHex for Covenant {
    type Error = DecodingError;

    fn from_hex<T: AsRef<[u8]>>(hex: T) -> std::result::Result<Self, Self::Error> {
        Covenant::decode(&mut Buffer::from_hex(hex)?)
    }
}

#[cfg(feature = "json")]
impl serde::Serialize for Covenant {
    fn serialize<S: serde::Serializer>(&self, s: S) -> std::result::Result<S::Ok, S::Error> {
        let mut state = s.serialize_struct("Covenant", 2)?;
        state.serialize_field("type", &self.get_type())?;
        state.serialize_field("hash", &self.get_action())?;
        state.serialize_field("items", &self.get_items())?;
        state.end()
    }
}

//TODO finish up testing the global functions.
#[cfg(test)]
mod tests {
    use super::*;
    use crate::Address;
    use extended_primitives::Hash;
    use handshake_types::{Name, NameHash};

    #[test]
    fn test_covenant_encoding() {
        let empty_cov = Covenant::None;

        assert_eq!(empty_cov.to_hex(), "0000");

        //@todo maybe move this into the claim file?
        let claim_cov = Covenant::Claim(ClaimCovenant {
            //@todo would be nice to just have this from name. So name.to_hash()
            name_hash: NameHash::from_hex(
                "7f092b58e32d1875652f36bdf2f5242ef2048dd8e5ff27988437c1c7aeda9007",
            )
            .unwrap(),
            height: 100,
            name: Name::from("satoshi".to_owned()),
            flags: 0,
            commit_hash: Hash::from_hex(
                "7f092b58e32d1875652f36bdf2f5242ef2048dd8e5ff27988437c1c7aeda9007",
            )
            .unwrap(),
            commit_height: 150,
        });

        assert_eq!(claim_cov.to_hex(), "0106207f092b58e32d1875652f36bdf2f5242ef2048dd8e5ff27988437c1c7aeda90070464000000077361746f7368690100207f092b58e32d1875652f36bdf2f5242ef2048dd8e5ff27988437c1c7aeda90070496000000");

        let open_cov = Covenant::Open(OpenCovenant {
            name_hash: NameHash::from_hex(
                "7f092b58e32d1875652f36bdf2f5242ef2048dd8e5ff27988437c1c7aeda9007",
            )
            .unwrap(),
            height: 0,
            name: Name::from("satoshi".to_owned()),
        });

        assert_eq!(open_cov.to_hex(), "0203207f092b58e32d1875652f36bdf2f5242ef2048dd8e5ff27988437c1c7aeda90070400000000077361746f736869");
        dbg!(open_cov.size());

        let bid_cov = Covenant::Bid(BidCovenant {
            name_hash: NameHash::from_hex(
                "7f092b58e32d1875652f36bdf2f5242ef2048dd8e5ff27988437c1c7aeda9007",
            )
            .unwrap(),
            height: 100,
            name: Name::from("satoshi".to_owned()),
            //@todo automatically generate this blind.
            hash: Hash::from_hex(
                "7f092b58e32d1875652f36bdf2f5242ef2048dd8e5ff27988437c1c7aeda9007",
            )
            .unwrap(),
        });

        assert_eq!(bid_cov.to_hex(), "0304207f092b58e32d1875652f36bdf2f5242ef2048dd8e5ff27988437c1c7aeda90070464000000077361746f736869207f092b58e32d1875652f36bdf2f5242ef2048dd8e5ff27988437c1c7aeda9007");

        let reveal_cov = Covenant::Reveal(RevealCovenant {
            name_hash: NameHash::from_hex(
                "7f092b58e32d1875652f36bdf2f5242ef2048dd8e5ff27988437c1c7aeda9007",
            )
            .unwrap(),
            height: 100,
            nonce: Hash::from_hex(
                "7f092b58e32d1875652f36bdf2f5242ef2048dd8e5ff27988437c1c7aeda9007",
            )
            .unwrap(),
        });

        assert_eq!(reveal_cov.to_hex(), "0403207f092b58e32d1875652f36bdf2f5242ef2048dd8e5ff27988437c1c7aeda90070464000000207f092b58e32d1875652f36bdf2f5242ef2048dd8e5ff27988437c1c7aeda9007");

        let redeem_cov = Covenant::Redeem(RedeemCovenant {
            name_hash: NameHash::from_hex(
                "7f092b58e32d1875652f36bdf2f5242ef2048dd8e5ff27988437c1c7aeda9007",
            )
            .unwrap(),
            height: 100,
        });

        assert_eq!(
            redeem_cov.to_hex(),
            "0502207f092b58e32d1875652f36bdf2f5242ef2048dd8e5ff27988437c1c7aeda90070464000000"
        );

        let register_cov = Covenant::Register(RegisterCovenant {
            name_hash: NameHash::from_hex(
                "7f092b58e32d1875652f36bdf2f5242ef2048dd8e5ff27988437c1c7aeda9007",
            )
            .unwrap(),
            height: 100,
            record_data: Buffer::from_hex("0000").unwrap(),
            block_hash: Hash::from_hex(
                "7f092b58e32d1875652f36bdf2f5242ef2048dd8e5ff27988437c1c7aeda9007",
            )
            .unwrap(),
        });

        assert_eq!(
            register_cov.to_hex(),
            "0604207f092b58e32d1875652f36bdf2f5242ef2048dd8e5ff27988437c1c7aeda90070464000000020000207f092b58e32d1875652f36bdf2f5242ef2048dd8e5ff27988437c1c7aeda9007"
        );

        let update_cov = Covenant::Update(UpdateCovenant {
            name_hash: NameHash::from_hex(
                "7f092b58e32d1875652f36bdf2f5242ef2048dd8e5ff27988437c1c7aeda9007",
            )
            .unwrap(),
            height: 100,
            record_data: Buffer::from_hex("0000").unwrap(),
        });

        assert_eq!(
            update_cov.to_hex(),
            "0703207f092b58e32d1875652f36bdf2f5242ef2048dd8e5ff27988437c1c7aeda90070464000000020000"
        );

        let renew_cov = Covenant::Renew(RenewCovenant {
            name_hash: NameHash::from_hex(
                "7f092b58e32d1875652f36bdf2f5242ef2048dd8e5ff27988437c1c7aeda9007",
            )
            .unwrap(),
            height: 100,
            block_hash: Hash::from_hex(
                "7f092b58e32d1875652f36bdf2f5242ef2048dd8e5ff27988437c1c7aeda9007",
            )
            .unwrap(),
        });

        assert_eq!(
            renew_cov.to_hex(),
            "0803207f092b58e32d1875652f36bdf2f5242ef2048dd8e5ff27988437c1c7aeda90070464000000207f092b58e32d1875652f36bdf2f5242ef2048dd8e5ff27988437c1c7aeda9007"
        );

        let transfer_cov = Covenant::Transfer(TransferCovenant {
            name_hash: NameHash::from_hex(
                "7f092b58e32d1875652f36bdf2f5242ef2048dd8e5ff27988437c1c7aeda9007",
            )
            .unwrap(),
            height: 100,
            version: 0,
            address: "hs1qd42hrldu5yqee58se4uj6xctm7nk28r70e84vx"
                .parse()
                .unwrap(),
        });

        assert_eq!(
            transfer_cov.to_hex(),
            "0904207f092b58e32d1875652f36bdf2f5242ef2048dd8e5ff27988437c1c7aeda900704640000000100146d5571fdbca1019cd0f0cd792d1b0bdfa7651c7e"
        );

        let finalize_cov = Covenant::Finalize(FinalizeCovenant {
            name_hash: NameHash::from_hex(
                "7f092b58e32d1875652f36bdf2f5242ef2048dd8e5ff27988437c1c7aeda9007",
            )
            .unwrap(),
            height: 100,
            name: Name::from("satoshi".to_owned()),
            flags: 0,
            claimed: 200,
            renewals: 300,
            block_hash: Hash::from_hex(
                "7f092b58e32d1875652f36bdf2f5242ef2048dd8e5ff27988437c1c7aeda9007",
            )
            .unwrap(),
        });

        assert_eq!(
            finalize_cov.to_hex(),
            "0a07207f092b58e32d1875652f36bdf2f5242ef2048dd8e5ff27988437c1c7aeda90070464000000077361746f736869010004c8000000042c010000207f092b58e32d1875652f36bdf2f5242ef2048dd8e5ff27988437c1c7aeda9007"
        );

        let revoke_cov = Covenant::Revoke(RevokeCovenant {
            name_hash: NameHash::from_hex(
                "7f092b58e32d1875652f36bdf2f5242ef2048dd8e5ff27988437c1c7aeda9007",
            )
            .unwrap(),
            height: 100,
        });

        assert_eq!(
            revoke_cov.to_hex(),
            "0b02207f092b58e32d1875652f36bdf2f5242ef2048dd8e5ff27988437c1c7aeda90070464000000"
        );
    }

    #[test]
    fn test_covenant_is_name() {
        let empty_cov = Covenant::None;

        assert!(!empty_cov.is_name());

        let bid = BidCovenant {
            name_hash: Default::default(),
            height: 0,
            name: Default::default(),
            hash: Default::default(),
        };

        let cov = Covenant::Bid(bid);

        assert!(cov.is_name());
    }
}
