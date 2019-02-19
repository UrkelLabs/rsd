use crypto::sha2::Sha256;

/// A Handshake covenant, which is a method of changing name state on the chain.
#[derive(PartialEq, Eq, Clone, Debug)]
pub enum Covenant {
    Empty,
    Claim,
    Bid,
    Open,
}

//TODO the methods that span all Covenants -> Like from, and various helpers
// impl Covenant {

// }

/// A Empty Covenant is included in the case of a payment.
/// It effects no change on the urkel tree.
/// In the original codebase the type of this covenant would be "none"
pub struct EmptyCovenant {
}

//TODO thought -> We could also make this a type of "NoneCovenant"

//TODO 
pub struct Claim {
    pub name_hash: Sha256,
    pub height: u32,
    pub name: String,
    //TODO verify type on this.
    pub flags: String
}

pub struct Open {
    ///The hash of the name for the Open.
    //check if this is the correct type for namehash.
    pub name_hash: Sha256,
    ///The height at which the bid occured
    ///The height should always be 0 for an Open.
    pub height: u32,
    ///The raw name that the open is for.
    pub name: String,
}

pub struct Bid {
    pub name_hash: Sha256,
    pub height: u32,
    pub name: String,
    pub hash: Sha256,
}

pub struct Reveal {
    pub name_hash: Sha256,
    pub height: u32,
    //TODO verify output type. This might also be better served as a Sha256
    pub nonce: String
}

pub struct Redeem {
    pub name_hash: Sha256,
    pub height: u32
}

pub struct Register {
    pub name_hash: Sha256,
    pub height: u32,
    //TODO verify type. I believe this is a serialized encoding of the data insert. We should make
    //this a cutom type.
    pub record_data: String,
    //TODO Verify type. Not going to be a Sha256 (I think) Since we are using a different hashing
    //method. Will have to grab a type from the crypto library when we finish it.
    pub block_hash: String
}

pub struct Update {
    pub name_hash: Sha256,
    pub height: u32,
    //TODO See Above.
    pub record_data: String,
    //TODO see above.
    pub block_hash: String,
}

pub struct Renew {
    pub name_hash: Sha256,
    pub height: u32,
    //TODO see above.
    pub block_hash: String,
}

pub struct Transfer {
    pub name_hash: Sha256,
    pub height: u32,
    //TODO verify type
    pub version: u32,
    //TODO convert this to Address type.
    pub address: String
}

pub struct Finalize {
    pub name_hash: Sha256,
    pub height: u32,
    //TODO this should be a custom type.
    pub name: String,
    //TODO see above.
    pub flags: String,
    //TODO see above
    pub block_hash: String,
}

pub struct Revoke {
    pub name_hash: Sha256,
    pub height: u32,
}

