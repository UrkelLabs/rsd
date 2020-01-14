use crate::block_template::json::BlockTemplateJSON;
use crate::{Address, Input, Output, Transaction};
use extended_primitives::{Buffer, Hash, Uint256};
use handshake_types::Time;
use hex::FromHex;
//@todo make a builder for block template, since so many of the options are likely not easy to do
//via just a simple new function. Have "new" cover the most basic of options, and then the builder
//does anything more extensive.
//
#[derive(Default)]
pub struct BlockTemplateBuilder {
    pub prev_block: Hash,
    pub version: u32,
    pub height: u32,
    pub time: Time,
    // TODO convert back to Compact type, but use u32 for now.
    pub bits: u32,
    pub target: Uint256,
    pub median_time: Time,
    //@todo see: https://github.com/handshake-org/hsd/blob/master/lib/blockchain/chain.js#L3480
    pub flags: u32,
    // To show who the block is mined by: eg. "Mined by Bitamin" see: https://github.com/handshake-org/hsd/blob/master/lib/mining/miner.js#L472
    //@todo Should default to "mined by RSD"
    pub coinbase_flags: String,
    //@fixme this should *not* be hash it should be address.
    pub address: Address,
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
    pub merkle_root: Hash,
    pub witness_root: Hash,
    pub tree_root: Hash,
    pub reserved_root: Hash,
    pub coinbase: Transaction,
    pub mask: Hash,
    pub transactions: Vec<Transaction>,
}

impl BlockTemplateBuilder {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn with_json(mut self, template: BlockTemplateJSON) -> Self {
        let mut bits_bytes = [0u8; 4];
        bits_bytes.copy_from_slice(&hex::decode(template.bits).unwrap());
        let bits = u32::from_be_bytes(bits_bytes);

        self.prev_block = template.previous_blockhash;
        self.version = template.version;
        self.height = template.height;
        self.time = Time::from(template.cur_time);
        self.bits = bits;
        self.target = template.target;
        //@todo smells, but just reversing this: https://github.com/handshake-org/hsd/blob/master/lib/node/rpc.js#L1486
        self.median_time = Time::from(template.min_time - 1);
        //@todo
        self.coinbase_flags = template.coinbase_aux.flags;
        self.sigop_limit = template.sig_op_limit;
        self.weight_limit = template.weight_limit;
        self.tree_root = template.tree_root;
        self.reserved_root = template.reserved_root;
        self.mask = template.mask;
        //This should really be like self.with_transactions(txs), so that we can fill out those
        //count numbers.
        // self.transactions = template.transactions;
        let mut txs = Vec::new();
        for tx in template.transactions.iter() {
            txs.push(tx.data.clone());
        }

        self.with_transactions_hex(txs)
    }

    pub fn with_transactions_hex(mut self, txs: Vec<String>) -> Self {
        let mut templte_txs = Vec::new();
        for tx in txs.iter() {
            //@todo need to add counts to self.
            templte_txs.push(Transaction::from_hex(&tx).unwrap());
        }

        self.transactions = templte_txs;
        self
    }
}
