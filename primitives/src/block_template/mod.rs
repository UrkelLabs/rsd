#[cfg(feature = "json")]
pub mod json;

#[cfg(feature = "json")]
use json::BlockTemplateJSON;

pub mod airdrop;
pub mod builder;
use crate::{Address, Input, Output, Transaction};
use airdrop::BlockAirdrop;
use cryptoxide::blake2b::Blake2b;
use cryptoxide::digest::Digest;
use extended_primitives::{Buffer, Hash, Uint256};

// use handshake_encoding::{Decodable, DecodingError, Encodable};
use handshake_protocol::consensus::get_reward;
// use handshake_script::Witness;
use handshake_types::{Amount, MerkleTree, Time};

//@todo -> maybe switch this to block/block_template.rs
//@todo -> implement defaults

//@todo wrap this inside a module for block.
pub struct BlockTemplate {
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
    //@todo need airdrop claim (sp)?
    // pub claims: Vec<AirdropClaim>,
    //@todo need airdrop proof type.
    // pub airdrops: Vec<AirdropProof>,
}

impl BlockTemplate {
    pub fn create_coinbase(&self) -> Transaction {
        let mut inputs = Vec::new();
        let mut outputs = Vec::new();

        // Commit to height
        let locktime = self.height;

        // Coinbase input
        let input = Input::new_coinbase(&self.coinbase_flags);
        inputs.push(input);

        //Reward output.
        let output = Output::new(self.get_reward(), self.address.clone());
        outputs.push(output);

        //@todo add claims and proofs
        //
        let cb = Transaction::new(locktime, inputs, outputs);

        //Not needed I believe. @todo
        // cb.refresh()

        // assert!(cb.inputs[0].witness.getSize() <= 1000);

        cb
    }

    pub fn mask_hash(&self) -> Hash {
        let mut sh = Blake2b::new(32);
        let mut output = [0; 32];
        sh.input(&self.prev_block.to_array());
        sh.input(&self.mask.to_array());
        sh.result(&mut output);

        Hash::from(output)
    }

    pub fn refresh(&mut self) {
        let cb = self.create_coinbase();

        let mut leaves = Vec::new();
        leaves.push(cb.hash());

        for tx in self.transactions.iter() {
            leaves.push(tx.hash());
        }

        self.merkle_root = MerkleTree::from_leaves(leaves).get_root();

        let mut leaves = Vec::new();
        leaves.push(cb.witness_hash());

        for tx in self.transactions.iter() {
            leaves.push(tx.witness_hash());
        }

        self.witness_root = MerkleTree::from_leaves(leaves).get_root();

        self.coinbase = cb;
    }

    pub fn get_reward(&self) -> Amount {
        //@todo
        // Amount::ZERO
        let reward = get_reward(self.height, self.interval);
        // reward + Amount::from_doos(self.fees as u64)
        reward
            .checked_add(Amount::from_doos(self.fees as u64))
            .unwrap() //@todo not sure best way to handle here.
    }
}

// impl Encodable for BlockTemplate {
//     fn size(&self) -> usize {
//         let size = 4
//             + self.time.size()
//             + 4
//             + 4
//             + 32
//             + self.median_time.size()
//             + 4
//             + 1
//             + self.coinbase_flags.len()
//             + 32
//             + 4
//             + 4
//             + 4
//             + 4
//             + 4
//             + 4
//             + 4
//             // + self.tree.size()
//             + 32
//             + 32
//             + 32
//             + 32
//             + self.right.len()
//             + self.left.len()
//             + 1;

//         for tx in self.transactions.iter() {
//             size += tx.size();
//         }

//         size
//     }

//     fn encode(&self) -> Buffer {
//         let mut buffer = Buffer::new();

//         buffer.write_u32(self.version);
//         buffer.extend(self.time.encode());
//         buffer.write_u32(self.height);
//         buffer.write_u32(self.bits);
//         buffer.write_u256(self.target);
//         buffer.extend(self.median_time.encode());
//         buffer.write_u32(self.flags);
//         buffer.write_u8(self.coinbase_flags.len() as u8);
//         buffer.write_str(&self.coinbase_flags);
//         buffer.extend(self.address.encode());
//         buffer.write_u32(self.sigop_limit);
//         buffer.write_u32(self.weight_limit);
//         buffer.write_u32(self.opens);
//         buffer.write_u32(self.updates);
//         buffer.write_u32(self.renewals);
//         buffer.write_u32(self.interval);
//         buffer.write_u32(self.fees);
//         // buffer.extend(self.tree.encode());
//         buffer.write_hash(self.previous_header_hash);
//         buffer.write_hash(self.tree_root);
//         buffer.write_hash(self.filter_root);
//         buffer.write_hash(self.reserved_root);
//         buffer.extend(self.right);
//         buffer.extend(self.left);

//         buffer.write_u8(self.transactions.len() as u8);

//         for tx in self.transactions.iter() {
//             buffer.extend(tx.encode());
//         }

//         buffer
//     }
// }

// impl Decodable for BlockTemplate {
//     type Err = DecodingError;

//     fn decode(buffer: &mut Buffer) -> Result<Self, Self::Err> {
//         let count = buffer.read_varint()?;
//         let mut txdata = Vec::new();

//         for _ in 0..count.as_u64() {
//             txdata.push(Transaction::decode(buffer)?);
//         }

//         let version = buffer.read_u32()?;
//         let time = Time::decode(buffer)?;
//         let height = buffer.read_u32()?;
//         let bits = buffer.read_u32()?;
//         // let target = buffer.read_
//         buffer.write_u256(self.target);
//         buffer.extend(self.median_time.encode());
//         buffer.write_u32(self.flags);
//         buffer.write_u8(self.coinbase_flags.len() as u8);
//         buffer.write_str(&self.coinbase_flags);
//         buffer.extend(self.address.encode());
//         buffer.write_u32(self.sigop_limit);
//         buffer.write_u32(self.weight_limit);
//         buffer.write_u32(self.opens);
//         buffer.write_u32(self.updates);
//         buffer.write_u32(self.renewals);
//         buffer.write_u32(self.interval);
//         buffer.write_u32(self.fees);
//         // buffer.extend(self.tree.encode());
//         buffer.write_hash(self.previous_header_hash);
//         buffer.write_hash(self.tree_root);
//         buffer.write_hash(self.filter_root);
//         buffer.write_hash(self.reserved_root);
//         buffer.extend(self.right);
//         buffer.extend(self.left);

//         buffer.write_u8(self.transactions.len() as u8);

//         for tx in self.transactions.iter() {
//             buffer.extend(tx.encode());
//         }

//         Ok(Block { header, txdata })
//     }
// }
