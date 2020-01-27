use extended_primitives::{Buffer, Hash, Uint256};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BlockTemplateJSON {
    //@todo custom type here.
    pub capabilities: Vec<String>,
    pub mutable: Vec<String>,
    pub version: u32,
    pub rules: Vec<String>,
    //Not going to work
    #[serde(rename = "vbavailable")]
    pub vb_available: VbAvailable,
    #[serde(rename = "vbrequired")]
    pub vb_required: u32,
    pub height: u32,
    #[serde(rename = "previousblockhash")]
    pub previous_blockhash: Hash,
    #[serde(rename = "treeroot")]
    pub tree_root: Hash,
    pub mask: Hash,
    #[serde(rename = "reservedroot")]
    pub reserved_root: Hash,
    pub target: Uint256,
    pub bits: String,
    #[serde(rename = "noncerange")]
    pub nonce_range: String,
    #[serde(rename = "curtime")]
    pub cur_time: u64,
    #[serde(rename = "mintime")]
    pub min_time: u64,
    #[serde(rename = "maxtime")]
    pub max_time: u64,
    pub expires: u64,
    #[serde(rename = "sigoplimit")]
    pub sig_op_limit: u32,
    #[serde(rename = "sizelimit")]
    pub size_limit: u64,
    #[serde(rename = "weightlimit")]
    pub weight_limit: u32,
    #[serde(rename = "longpollid")]
    pub long_poll_id: String,
    #[serde(rename = "submitold")]
    pub submit_old: bool,
    #[serde(rename = "coinbaseaux")]
    pub coinbase_aux: CoinbaseAux,
    #[serde(rename = "coinbasevalue")]
    pub coinbase_value: u64,
    pub claims: Vec<ClaimEntry>,
    pub airdrops: Vec<AirdropEntry>,
    pub transactions: Vec<TransactionEntry>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct VbAvailable {}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CoinbaseAux {
    pub flags: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TransactionEntry {
    pub data: Buffer,
    pub txid: Hash,
    pub hash: Hash,
    pub depends: Vec<Hash>,
    pub fee: u32,
    pub sigops: u32,
    pub weight: u32,
}

//@todo everything that is Hex should probably be a string.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ClaimEntry {
    data: String,
    name: String,
    name_hash: Hash,
    version: u8,
    hash: Hash,
    //@todo check type here.
    value: u32,
    //@todo check type here.
    fee: u32,
    weak: bool,
    commit_hash: Hash,
    commit_height: u32,
    //@todo check type here.
    weight: u32,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AirdropEntry {
    data: String,
    //@todo check all these types
    position: u32,
    version: u8,
    //@todo see the best way to parse this from the address hash.
    address: String,
    //@todo check all these types
    value: u32,
    fee: u32,
    rate: u32,
    weak: bool,
}
