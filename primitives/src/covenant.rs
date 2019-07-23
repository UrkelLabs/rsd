use crate::Address;
use extended_primitives::VarInt;
use extended_primitives::{Buffer, Hash, Uint256};
use handshake_protocol::encoding::{Decodable, DecodingError, Encodable};
use handshake_types::{Name, NameHash};

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
}

impl Decodable for Covenant {
    type Error = DecodingError;

    fn decode(buffer: &mut Buffer) -> Result<Self, Self::Error> {
        let covenant_type = buffer.read_u8()?;

        match covenant_type {
            0 => Ok(Covenant::None),
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
    fn size(&self) -> u32 {
        match self {
            Covenant::None => 0,
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
        }
    }

    fn encode(&self) -> Buffer {
        match self {
            Covenant::None => Buffer::new(),
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

//TODO formatting, and I think common functions to_hex, from_hex.
//when I say formatting I mean Debug and to_string functions.

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ClaimCovenant {
    pub name_hash: NameHash,
    pub height: u32,
    pub name: Name,
    pub flags: String,
}

impl Encodable for ClaimCovenant {
    fn size(&self) -> u32 {
        let mut size = 0;
        let name_hash_length = VarInt::from(32 as u64);
        let height_length = VarInt::from(4 as u64);
        let name_length = VarInt::from(self.name.len() as u64);
        let flags_length = VarInt::from(self.flags.len() as u64);

        size += name_hash_length.encoded_size();
        size += height_length.encoded_size();
        size += name_length.encoded_size();
        size += flags_length.encoded_size();

        size
    }

    fn encode(&self) -> Buffer {
        let mut buffer = Buffer::new();

        buffer.write_u8(1);
        buffer.write_varint(4);

        //Name Hash
        //Hashes are 32 bytes
        buffer.write_varint(32);
        buffer.write_hash(self.name_hash);

        //Height
        buffer.write_varint(4);
        buffer.write_u32(self.height);

        //Name
        buffer.write_varint(self.name.len());
        buffer.write_str(&self.name);

        //Flags
        buffer.write_varint(self.flags.len());
        buffer.write_str(&self.flags);

        buffer
    }
}

impl Decodable for ClaimCovenant {
    type Error = DecodingError;

    fn decode(buffer: &mut Buffer) -> Result<Self, Self::Error> {
        //4
        buffer.read_varint()?;

        buffer.read_varint()?;
        let name_hash = buffer.read_hash()?;

        buffer.read_varint()?;
        let height = buffer.read_u32()?;

        let name_length = buffer.read_varint()?;
        //TODO check
        let name = buffer.read_string(name_length.as_u64() as usize)?;

        let flags_length = buffer.read_varint()?;
        let flags = buffer.read_string(flags_length.as_u64() as usize)?;

        Ok(ClaimCovenant {
            name_hash,
            height,
            name: Name::from(name),
            flags,
        })
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct OpenCovenant {
    ///The hash of the name for the Open.
    pub name_hash: NameHash,
    ///The height at which the bid occured
    ///The height should always be 0 for an Open.
    pub height: u32,
    ///The raw name that the open is for.
    pub name: Name,
}

impl Encodable for OpenCovenant {
    fn size(&self) -> u32 {
        let mut size = 0;
        let name_hash_length = VarInt::from(32 as u64);
        let height_length = VarInt::from(4 as u64);
        let name_length = VarInt::from(self.name.len() as u64);

        size += name_hash_length.encoded_size();
        size += height_length.encoded_size();
        size += name_length.encoded_size();

        size
    }

    fn encode(&self) -> Buffer {
        let mut buffer = Buffer::new();

        buffer.write_u8(2);
        buffer.write_varint(3);

        //Name Hash
        //Hashes are 32 bytes
        buffer.write_varint(32);
        buffer.write_hash(self.name_hash);

        //Height
        buffer.write_varint(4);
        buffer.write_u32(self.height);

        //Name
        buffer.write_varint(self.name.len());
        buffer.write_str(&self.name);

        buffer
    }
}

impl Decodable for OpenCovenant {
    type Error = DecodingError;

    fn decode(buffer: &mut Buffer) -> Result<Self, Self::Error> {
        //3
        buffer.read_varint()?;

        buffer.read_varint()?;
        let name_hash = buffer.read_hash()?;

        buffer.read_varint()?;
        let height = buffer.read_u32()?;

        let name_length = buffer.read_varint()?;
        //TODO check
        let name = buffer.read_string(name_length.as_u64() as usize)?;

        Ok(OpenCovenant {
            name_hash,
            height,
            name: Name::from(name),
        })
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct BidCovenant {
    pub name_hash: NameHash,
    pub height: u32,
    pub name: Name,
    //TODO *might* want to make this a BidHash, but that'll be a later impl
    pub hash: Hash,
}

impl Encodable for BidCovenant {
    fn size(&self) -> u32 {
        let mut size = 0;
        let name_hash_length = VarInt::from(32 as u64);
        let height_length = VarInt::from(4 as u64);
        let name_length = VarInt::from(self.name.len() as u64);
        let hash_length = VarInt::from(32 as u64);

        size += name_hash_length.encoded_size();
        size += height_length.encoded_size();
        size += name_length.encoded_size();
        size += hash_length.encoded_size();

        size
    }

    fn encode(&self) -> Buffer {
        let mut buffer = Buffer::new();

        buffer.write_u8(3);
        buffer.write_varint(4);

        //Name Hash
        //Hashes are 32 bytes
        buffer.write_varint(32);
        buffer.write_hash(self.name_hash);

        //Height
        buffer.write_varint(4);
        buffer.write_u32(self.height);

        //Name
        buffer.write_varint(self.name.len());
        buffer.write_str(&self.name);

        //Hashes are 32 bytes
        buffer.write_varint(32);
        buffer.write_hash(self.hash);

        buffer
    }
}

impl Decodable for BidCovenant {
    type Error = DecodingError;

    fn decode(buffer: &mut Buffer) -> Result<Self, Self::Error> {
        //4
        buffer.read_varint()?;

        buffer.read_varint()?;
        let name_hash = buffer.read_hash()?;

        buffer.read_varint()?;
        let height = buffer.read_u32()?;

        let name_length = buffer.read_varint()?;
        //TODO check
        let name = buffer.read_string(name_length.as_u64() as usize)?;

        buffer.read_varint()?;
        let hash = buffer.read_hash()?;

        Ok(BidCovenant {
            name_hash,
            height,
            name: Name::from(name),
            hash,
        })
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct RevealCovenant {
    pub name_hash: NameHash,
    pub height: u32,
    pub nonce: Uint256,
}

impl Encodable for RevealCovenant {
    fn size(&self) -> u32 {
        let mut size = 0;
        let name_hash_length = VarInt::from(32 as u64);
        let height_length = VarInt::from(4 as u64);
        //TODO double check this.
        let nonce_length = VarInt::from(32 as u64);

        size += name_hash_length.encoded_size();
        size += height_length.encoded_size();
        size += nonce_length.encoded_size();

        size
    }

    fn encode(&self) -> Buffer {
        let mut buffer = Buffer::new();

        buffer.write_u8(4);
        buffer.write_varint(3);

        //Name Hash
        //Hashes are 32 bytes
        buffer.write_varint(32);
        buffer.write_hash(self.name_hash);

        //Height
        buffer.write_varint(4);
        buffer.write_u32(self.height);

        //Nonce
        buffer.write_varint(32);
        buffer.write_u256(self.nonce);

        buffer
    }
}

impl Decodable for RevealCovenant {
    type Error = DecodingError;

    fn decode(buffer: &mut Buffer) -> Result<Self, Self::Error> {
        //3
        buffer.read_varint()?;

        buffer.read_varint()?;
        let name_hash = buffer.read_hash()?;

        buffer.read_varint()?;
        let height = buffer.read_u32()?;

        buffer.read_varint()?;
        let nonce = buffer.read_u256()?;

        Ok(RevealCovenant {
            name_hash,
            height,
            nonce,
        })
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct RedeemCovenant {
    pub name_hash: NameHash,
    pub height: u32,
}

impl Encodable for RedeemCovenant {
    fn size(&self) -> u32 {
        let mut size = 0;
        let name_hash_length = VarInt::from(32 as u64);
        let height_length = VarInt::from(4 as u64);

        size += name_hash_length.encoded_size();
        size += height_length.encoded_size();

        size
    }

    fn encode(&self) -> Buffer {
        let mut buffer = Buffer::new();

        buffer.write_u8(5);
        buffer.write_varint(2);

        //Name Hash
        //Hashes are 32 bytes
        buffer.write_varint(32);
        buffer.write_hash(self.name_hash);

        //Height
        buffer.write_varint(4);
        buffer.write_u32(self.height);

        buffer
    }
}

impl Decodable for RedeemCovenant {
    type Error = DecodingError;

    fn decode(buffer: &mut Buffer) -> Result<Self, Self::Error> {
        //2
        buffer.read_varint()?;

        buffer.read_varint()?;
        let name_hash = buffer.read_hash()?;

        buffer.read_varint()?;
        let height = buffer.read_u32()?;

        Ok(RedeemCovenant { name_hash, height })
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct RegisterCovenant {
    pub name_hash: NameHash,
    pub height: u32,
    //TODO verify type. I believe this is a serialized encoding of the data insert. We should make
    //this a cutom type.
    pub record_data: String,
    //TODO not sure if we want to have BlockHash custom type, but it might make sense.
    //Otherwise Hash here is just fine
    pub block_hash: Hash,
}

impl Encodable for RegisterCovenant {
    fn size(&self) -> u32 {
        let mut size = 0;
        let name_hash_length = VarInt::from(32 as u64);
        let height_length = VarInt::from(4 as u64);
        let name_length = VarInt::from(self.record_data.len() as u64);
        let block_length = VarInt::from(32 as u64);

        size += name_hash_length.encoded_size();
        size += height_length.encoded_size();
        size += name_length.encoded_size();
        size += block_length.encoded_size();

        size
    }

    fn encode(&self) -> Buffer {
        let mut buffer = Buffer::new();

        buffer.write_u8(6);
        buffer.write_varint(4);

        //Name Hash
        //Hashes are 32 bytes
        buffer.write_varint(32);
        buffer.write_hash(self.name_hash);

        //Height
        buffer.write_varint(4);
        buffer.write_u32(self.height);

        //Record Data
        buffer.write_varint(self.record_data.len());
        buffer.write_str(&self.record_data);

        //Block Hash
        buffer.write_varint(32);
        buffer.write_hash(self.block_hash);

        buffer
    }
}

impl Decodable for RegisterCovenant {
    type Error = DecodingError;

    fn decode(buffer: &mut Buffer) -> Result<Self, Self::Error> {
        //4
        buffer.read_varint()?;

        buffer.read_varint()?;
        let name_hash = buffer.read_hash()?;

        buffer.read_varint()?;
        let height = buffer.read_u32()?;

        //Record Data
        let record_length = buffer.read_varint()?;
        let record_data = buffer.read_string(record_length.as_u64() as usize)?;

        buffer.read_varint()?;
        let block_hash = buffer.read_hash()?;

        Ok(RegisterCovenant {
            name_hash,
            height,
            record_data,
            block_hash,
        })
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct UpdateCovenant {
    pub name_hash: NameHash,
    pub height: u32,
    //TODO See Above.
    pub record_data: String,
    //TODO see above.
    pub block_hash: Hash,
}

impl Encodable for UpdateCovenant {
    fn size(&self) -> u32 {
        let mut size = 0;
        let name_hash_length = VarInt::from(32 as u64);
        let height_length = VarInt::from(4 as u64);
        let name_length = VarInt::from(self.record_data.len() as u64);
        let block_length = VarInt::from(32 as u64);

        size += name_hash_length.encoded_size();
        size += height_length.encoded_size();
        size += name_length.encoded_size();
        size += block_length.encoded_size();

        size
    }

    fn encode(&self) -> Buffer {
        let mut buffer = Buffer::new();

        buffer.write_u8(7);
        buffer.write_varint(4);

        //Name Hash
        //Hashes are 32 bytes
        buffer.write_varint(32);
        buffer.write_hash(self.name_hash);

        //Height
        buffer.write_varint(4);
        buffer.write_u32(self.height);

        //Record Data
        buffer.write_varint(self.record_data.len());
        buffer.write_str(&self.record_data);

        //Block Hash
        buffer.write_varint(32);
        buffer.write_hash(self.block_hash);

        buffer
    }
}

impl Decodable for UpdateCovenant {
    type Error = DecodingError;

    fn decode(buffer: &mut Buffer) -> Result<Self, Self::Error> {
        //4
        buffer.read_varint()?;

        buffer.read_varint()?;
        let name_hash = buffer.read_hash()?;

        buffer.read_varint()?;
        let height = buffer.read_u32()?;

        //Record Data
        let record_length = buffer.read_varint()?;
        let record_data = buffer.read_string(record_length.as_u64() as usize)?;

        buffer.read_varint()?;
        let block_hash = buffer.read_hash()?;

        Ok(UpdateCovenant {
            name_hash,
            height,
            record_data,
            block_hash,
        })
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct RenewCovenant {
    pub name_hash: NameHash,
    pub height: u32,
    //TODO see above.
    pub block_hash: Hash,
}

impl Encodable for RenewCovenant {
    fn size(&self) -> u32 {
        let mut size = 0;
        let name_hash_length = VarInt::from(32 as u64);
        let height_length = VarInt::from(4 as u64);
        let block_length = VarInt::from(32 as u64);

        size += name_hash_length.encoded_size();
        size += height_length.encoded_size();
        size += block_length.encoded_size();

        size
    }

    fn encode(&self) -> Buffer {
        let mut buffer = Buffer::new();

        buffer.write_u8(8);
        buffer.write_varint(3);

        //Name Hash
        //Hashes are 32 bytes
        buffer.write_varint(32);
        buffer.write_hash(self.name_hash);

        //Height
        buffer.write_varint(4);
        buffer.write_u32(self.height);

        //Block Hash
        buffer.write_varint(32);
        buffer.write_hash(self.block_hash);

        buffer
    }
}

impl Decodable for RenewCovenant {
    type Error = DecodingError;

    fn decode(buffer: &mut Buffer) -> Result<Self, Self::Error> {
        //3
        buffer.read_varint()?;

        buffer.read_varint()?;
        let name_hash = buffer.read_hash()?;

        buffer.read_varint()?;
        let height = buffer.read_u32()?;

        buffer.read_varint()?;
        let block_hash = buffer.read_hash()?;

        Ok(RenewCovenant {
            name_hash,
            height,
            block_hash,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TransferCovenant {
    pub name_hash: NameHash,
    pub height: u32,
    pub version: u32,
    pub address: Address,
}

impl Encodable for TransferCovenant {
    fn size(&self) -> u32 {
        let mut size = 0;
        //TODO because all these values are below 252
        let name_hash_length = VarInt::from(32 as u64);
        let height_length = VarInt::from(4 as u64);
        let version_length = VarInt::from(4 as u64);
        let address_length = VarInt::from(self.address.size() as u64);

        size += name_hash_length.encoded_size();
        size += height_length.encoded_size();
        size += version_length.encoded_size();
        size += address_length.encoded_size();

        size
    }

    fn encode(&self) -> Buffer {
        let mut buffer = Buffer::new();

        buffer.write_u8(9);
        buffer.write_varint(4);

        //Name Hash
        //Hashes are 32 bytes
        buffer.write_varint(32);
        buffer.write_hash(self.name_hash);

        //Height
        buffer.write_varint(4);
        buffer.write_u32(self.height);

        //Version
        buffer.write_varint(4);
        buffer.write_u32(self.version);

        //Block Hash
        buffer.write_varint(self.address.size() as usize);
        buffer.extend(self.address.encode());

        buffer
    }
}

impl Decodable for TransferCovenant {
    type Error = DecodingError;

    fn decode(buffer: &mut Buffer) -> Result<Self, Self::Error> {
        //4
        buffer.read_varint()?;

        buffer.read_varint()?;
        let name_hash = buffer.read_hash()?;

        buffer.read_varint()?;
        let height = buffer.read_u32()?;

        buffer.read_varint()?;
        let version = buffer.read_u32()?;

        buffer.read_varint()?;
        let address = Address::decode(buffer)?;

        Ok(TransferCovenant {
            name_hash,
            height,
            version,
            address,
        })
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct FinalizeCovenant {
    pub name_hash: NameHash,
    pub height: u32,
    pub name: Name,
    //TODO see above.
    pub flags: String,
    //TODO see above
    pub block_hash: Hash,
}

impl Encodable for FinalizeCovenant {
    fn size(&self) -> u32 {
        let mut size = 0;
        let name_hash_length = VarInt::from(32 as u64);
        let height_length = VarInt::from(4 as u64);
        let name_length = VarInt::from(self.name.len() as u64);
        let flags_length = VarInt::from(self.flags.len() as u64);
        let block_hash_length = VarInt::from(32 as u64);

        size += name_hash_length.encoded_size();
        size += height_length.encoded_size();
        size += name_length.encoded_size();
        size += flags_length.encoded_size();
        size += block_hash_length.encoded_size();

        size
    }

    fn encode(&self) -> Buffer {
        let mut buffer = Buffer::new();

        buffer.write_u8(10);
        buffer.write_varint(5);

        //Name Hash
        //Hashes are 32 bytes
        buffer.write_varint(32);
        buffer.write_hash(self.name_hash);

        //Height
        buffer.write_varint(4);
        buffer.write_u32(self.height);

        //Record Data
        buffer.write_varint(self.name.len());
        buffer.write_str(&self.name);

        //Record Data
        buffer.write_varint(self.flags.len());
        buffer.write_str(&self.flags);

        //Block Hash
        buffer.write_varint(32);
        buffer.write_hash(self.block_hash);

        buffer
    }
}

impl Decodable for FinalizeCovenant {
    type Error = DecodingError;

    fn decode(buffer: &mut Buffer) -> Result<Self, Self::Error> {
        //5
        buffer.read_varint()?;

        buffer.read_varint()?;
        let name_hash = buffer.read_hash()?;

        buffer.read_varint()?;
        let height = buffer.read_u32()?;

        //Name
        let name_length = buffer.read_varint()?;
        let name = buffer.read_string(name_length.as_u64() as usize)?;

        //Flags
        let flags_length = buffer.read_varint()?;
        let flags = buffer.read_string(flags_length.as_u64() as usize)?;

        buffer.read_varint()?;
        let block_hash = buffer.read_hash()?;

        Ok(FinalizeCovenant {
            name_hash,
            height,
            name: Name::from(name),
            flags,
            block_hash,
        })
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct RevokeCovenant {
    pub name_hash: NameHash,
    pub height: u32,
}

impl Encodable for RevokeCovenant {
    fn size(&self) -> u32 {
        let mut size = 0;
        let name_hash_length = VarInt::from(32 as u64);
        let height_length = VarInt::from(4 as u64);

        size += name_hash_length.encoded_size();
        size += height_length.encoded_size();

        size
    }

    fn encode(&self) -> Buffer {
        let mut buffer = Buffer::new();

        buffer.write_u8(11);
        buffer.write_varint(2);

        //Name Hash
        //Hashes are 32 bytes
        buffer.write_varint(32);
        buffer.write_hash(self.name_hash);

        //Height
        buffer.write_varint(4);
        buffer.write_u32(self.height);

        buffer
    }
}

impl Decodable for RevokeCovenant {
    type Error = DecodingError;

    fn decode(buffer: &mut Buffer) -> Result<Self, Self::Error> {
        //2
        buffer.read_varint()?;

        buffer.read_varint()?;
        let name_hash = buffer.read_hash()?;

        buffer.read_varint()?;
        let height = buffer.read_u32()?;

        Ok(RevokeCovenant { name_hash, height })
    }
}

//TODO finish up testing the global functions.
#[cfg(test)]
mod tests {
    use super::*;

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
