use crate::Address;
use crate::{Hash, Name, NameHash, Uint256};

/// A Handshake covenant, which is a method of changing name state on the chain.
#[derive(PartialEq, Eq, Clone, Debug)]
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

//TODO formatting, and I think common functions to_hex, from_hex.
//when I say formatting I mean Debug and to_string functions.

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ClaimCovenant {
    pub name_hash: NameHash,
    pub height: u32,
    pub name: Name,
    pub flags: String,
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

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct BidCovenant {
    pub name_hash: NameHash,
    pub height: u32,
    pub name: Name,
    //TODO *might* want to make this a BidHash, but that'll be a later impl
    pub hash: Hash,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct RevealCovenant {
    pub name_hash: NameHash,
    pub height: u32,
    pub nonce: Uint256,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct RedeemCovenant {
    pub name_hash: NameHash,
    pub height: u32,
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

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct UpdateCovenant {
    pub name_hash: NameHash,
    pub height: u32,
    //TODO See Above.
    pub record_data: String,
    //TODO see above.
    pub block_hash: Hash,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct RenewCovenant {
    pub name_hash: NameHash,
    pub height: u32,
    //TODO see above.
    pub block_hash: Hash,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct TransferCovenant {
    pub name_hash: NameHash,
    pub height: u32,
    pub version: u32,
    pub address: Address,
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

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct RevokeCovenant {
    pub name_hash: NameHash,
    pub height: u32,
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
