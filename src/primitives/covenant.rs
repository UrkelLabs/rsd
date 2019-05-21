use crate::Address;
use crate::{Hash, Name, NameHash, Uint256};

/// A Handshake covenant, which is a method of changing name state on the chain.
#[derive(PartialEq, Eq, Clone, Debug)]
pub enum Covenant {
    Empty(EmptyCovenant),
    Claim(ClaimCovenant),
    Bid(BidCovenant),
    Open(OpenCovenant),
    Reveal(RevealCovenant),
    Register(RegisterCovenant),
}

//TODO the methods that span all Covenants -> Like from, and various helpers
// impl Covenant {

// }

/// A Empty Covenant is included in the case of a payment.
/// It effects no change on the urkel tree.
/// In the original codebase the type of this covenant would be "none"
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct EmptyCovenant {}

//TODO thought -> We could also make this a type of "NoneCovenant"

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

pub struct Update {
    pub name_hash: Blake2b,
    pub height: u32,
    //TODO See Above.
    pub record_data: String,
    //TODO see above.
    pub block_hash: String,
}

pub struct Renew {
    pub name_hash: Blake2b,
    pub height: u32,
    //TODO see above.
    pub block_hash: String,
}

pub struct Transfer {
    pub name_hash: Blake2b,
    pub height: u32,
    //TODO verify type
    pub version: u32,
    pub address: Address,
}

pub struct Finalize {
    pub name_hash: Blake2b,
    pub height: u32,
    //TODO this should be a custom type.
    pub name: String,
    //TODO see above.
    pub flags: String,
    //TODO see above
    pub block_hash: String,
}

pub struct Revoke {
    pub name_hash: Blake2b,
    pub height: u32,
}
