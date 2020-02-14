use crate::block_template::airdrop::BlockAirdrop;
use crate::block_template::json::BlockTemplateJSON;
use crate::{Address, BlockTemplate, Input, Output, Transaction};
use encodings::FromHex;
use extended_primitives::{Buffer, Hash, Uint256};
use handshake_encoding::Decodable;
use handshake_protocol::consensus::get_reward;
use handshake_types::{Amount, MerkleTree, Time};
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
    pub fees: u64,
    pub merkle_root: Hash,
    pub witness_root: Hash,
    pub tree_root: Hash,
    pub reserved_root: Hash,
    pub coinbase: Transaction,
    pub mask: Hash,
    pub transactions: Vec<Transaction>,
    pub airdrops: Vec<BlockAirdrop>,
}

impl BlockTemplateBuilder {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn with_json(mut self, template: BlockTemplateJSON) -> Self {
        let mut bits_bytes = [0u8; 4];
        bits_bytes.copy_from_slice(&Vec::from_hex(template.bits).unwrap());
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
            self.fees += tx.fee as u64;

            txs.push(tx.data.clone());
        }

        let mut airdrops = Vec::new();
        for airdrop in template.airdrops.iter() {
            self.fees += airdrop.fee;

            airdrops.push(BlockAirdrop::from_entry(airdrop.clone()));
        }

        self.airdrops = airdrops;

        self.with_transactions_hex(txs)
    }

    pub fn with_transactions_hex(mut self, txs: Vec<Buffer>) -> Self {
        let mut templte_txs = Vec::new();
        for tx in txs.iter() {
            //@todo need to add counts to self.
            //@todo also need to add fees to self as well.
            templte_txs.push(Transaction::decode(&mut tx.clone()).unwrap());
        }

        self.transactions = templte_txs;
        self
    }

    pub fn with_address(mut self, address: Address) -> Self {
        self.address = address;
        self
    }

    pub fn with_create_coinbase(mut self) -> Self {
        let mut inputs = Vec::new();
        let mut outputs = Vec::new();

        // Commit to height
        let locktime = self.height;

        // Coinbase input
        let input = Input::new_coinbase(&self.coinbase_flags);
        inputs.push(input);

        //Reward output.
        let output = Output::new(
            calculate_reward(self.height, self.interval, self.fees),
            self.address.clone(),
        );
        outputs.push(output);

        for proof in self.airdrops.iter() {
            dbg!("1");
            let input = Input::new_airdrop(&proof.blob);
            dbg!("2");
            inputs.push(input);

            dbg!("3");
            let output = Output::new(
                Amount::from_doos(proof.value - proof.fee),
                proof.address.clone(),
            );
            dbg!("4");
            outputs.push(output);
        }

        //@todo add claims
        self.coinbase = Transaction::new(locktime, inputs, outputs);

        //Not needed I believe. @todo
        // cb.refresh()

        // assert!(cb.inputs[0].witness.getSize() <= 1000);
        self
    }

    pub fn with_create_merkle_root(mut self) -> Self {
        let mut leaves = Vec::new();
        leaves.push(self.coinbase.hash());

        for tx in self.transactions.iter() {
            leaves.push(tx.hash());
        }

        self.merkle_root = MerkleTree::from_leaves(leaves).get_root();
        self
    }

    pub fn with_create_witness_root(mut self) -> Self {
        let mut leaves = Vec::new();
        leaves.push(self.coinbase.witness_hash());

        for tx in self.transactions.iter() {
            leaves.push(tx.witness_hash());
        }

        self.witness_root = MerkleTree::from_leaves(leaves).get_root();
        self
    }

    pub fn build(self) -> BlockTemplate {
        BlockTemplate {
            prev_block: self.prev_block,
            version: self.version,
            height: self.height,
            time: self.time,
            bits: self.bits,
            target: self.target,
            median_time: self.median_time,
            flags: self.flags,
            coinbase_flags: self.coinbase_flags,
            address: self.address,
            sigop_limit: self.sigop_limit,
            weight_limit: self.weight_limit,
            opens: self.opens,
            updates: self.updates,
            renewals: self.renewals,
            interval: self.interval,
            fees: self.fees,
            merkle_root: self.merkle_root,
            witness_root: self.witness_root,
            tree_root: self.tree_root,
            reserved_root: self.reserved_root,
            coinbase: self.coinbase,
            mask: self.mask,
            transactions: self.transactions,
            airdrops: self.airdrops,
        }
    }
}

pub fn calculate_reward(height: u32, _interval: u32, fees: u64) -> Amount {
    let reward = get_reward(height, 170_000);
    // reward + Amount::from_doos(self.fees as u64)
    reward.checked_add(Amount::from_doos(fees)).unwrap() //@todo not sure best way to handle here.
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_get_reward() {
        let reward = calculate_reward(391, 0, 0);
    }
}
