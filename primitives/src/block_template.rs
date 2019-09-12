use crate::Transaction;
use extended_primitives::{Buffer, Hash, Uint256};
use handshake_types::{MerkleTree, Time};

//@todo move this to Handshake primitives - Is for better organizational structure.
pub struct BlockTemplate {
    /// Version
    pub version: u32,
    pub time: Time,
    /// Block height
    pub height: u32,
    // /// The compressed difficulty
    // TODO convert back to Compact type, but use u32 for now.
    pub bits: u32,
    pub target: Uint256,
    pub median_time: Time,
    pub flags: u32,
    //TODO see: https://github.com/handshake-org/hsd/blob/master/lib/blockchain/chain.js#L3480
    // pub flags:
    // To show who the block is mined by: eg. "Mined by Bitamin" see: https://github.com/handshake-org/hsd/blob/master/lib/mining/miner.js#L472
    // Should default to "mined by RSD" TODO
    pub coinbase_flags: String,
    pub address: Hash,
    // /// Number of sigops allowed in the block
    pub sigop_limit: u32,
    pub weight_limit: u32,
    pub opens: u32,
    pub updates: u32,
    pub renewals: u32,
    //@todo should probably come from network constants.
    pub interval: u32,
    //@todo Probably move to Amount type.
    pub fees: u32,
    pub tree: MerkleTree,
    //@todo these will all be semi-switch in new pow.
    pub previous_header_hash: Hash,
    pub tree_root: Hash,
    pub filter_root: Hash,
    pub reserved_root: Hash,
    pub right: Buffer,
    pub left: Buffer,
    //@todo remove this for new pow.
    pub transactions: Vec<Transaction>,
    //@todo need airdrop claim (sp)?
    // pub claims: Vec<AirdropClaim>,
    //@todo need airdrop proof type.
    // pub airdrops: Vec<AirdropProof>,
}

//@todo maybe include.
///// Total funds available for the coinbase (in Satoshis)
//pub coinbase_value: u64,
////TODO figure out if all of these are needed or not.
//// /// Number of bytes allowed in the block
//pub size_limit: u32,

impl BlockTemplate {
    //Make Value a custom type here...
    pub fn get_reward(&self) -> u64 {
        0
    }
}
