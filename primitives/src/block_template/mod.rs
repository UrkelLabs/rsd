use crate::{Address, Transaction, Input, Output};
use handshake_script::Witness;
use extended_primitives::{Buffer, Hash, Uint256};
use handshake_encoding::{Decodable, DecodingError, Encodable};
use handshake_types::{MerkleTree, Time, Amount};
use handshake_protocol::consensus::get_reward;

//@todo -> maybe switch this to block/block_template.rs
//@todo -> implement defaults

//@todo wrap this inside a module for block.
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

    pub fn create_coinbase(&self) -> Transaction {
        let mut inputs = Vec::new();
        let mut outputs = Vec::new();

        // Commit to height
        let locktime = self.height;

        // Coinbase input
        let input = Input::new_coinbase(&self.coinbase_flags);
        inputs.push(input);

        //Reward output.
        let output = Output::new(self.get_reward(), self.address);
        outputs.push(output);


        //@todo add claims and proofs
        //
        let cb = Transaction::new(locktime, inputs, outputs);

        //Not needed I believe. @todo
        // cb.refresh()

        // assert!(cb.inputs[0].witness.getSize() <= 1000);

        cb
    }

    //Make Value a custom type here...
    pub fn get_reward(&self) -> Amount {
        //@todo
        // Amount::ZERO
        let reward = get_reward(self.height, self.interval);
        Amount::from_doo(reward) + Amount::from_doo(self.fees as u64)
    }
}

impl Encodable for BlockTemplate {
    fn size(&self) -> usize {
        let size = 4
            + self.time.size()
            + 4
            + 4
            + 32
            + self.median_time.size()
            + 4
            + 1
            + self.coinbase_flags.len()
            + 32
            + 4
            + 4
            + 4
            + 4
            + 4
            + 4
            + 4
            + self.tree.size()
            + 32
            + 32
            + 32
            + 32
            + self.right.len()
            + self.left.len()
            + 1;

        for tx in self.transactions.iter() {
            size += tx.size();
        }

        size
    }

    fn encode(&self) -> Buffer {
        let mut buffer = Buffer::new();

        buffer.write_u32(self.version);
        buffer.extend(self.time.encode());
        buffer.write_u32(self.height);
        buffer.write_u32(self.bits);
        buffer.write_u256(self.target);
        buffer.extend(self.median_time.encode());
        buffer.write_u32(self.flags);
        buffer.write_u8(self.coinbase_flags.len() as u8);
        buffer.write_str(&self.coinbase_flags);
        buffer.extend(self.address.encode());
        buffer.write_u32(self.sigop_limit);
        buffer.write_u32(self.weight_limit);
        buffer.write_u32(self.opens);
        buffer.write_u32(self.updates);
        buffer.write_u32(self.renewals);
        buffer.write_u32(self.interval);
        buffer.write_u32(self.fees);
        buffer.extend(self.tree.encode());
        buffer.write_hash(self.previous_header_hash);
        buffer.write_hash(self.tree_root);
        buffer.write_hash(self.filter_root);
        buffer.write_hash(self.reserved_root);
        buffer.extend(self.right);
        buffer.extend(self.left);

        buffer.write_u8(self.transactions.len() as u8);

        for tx in self.transactions.iter() {
            buffer.extend(tx.encode());
        }

        buffer
    }
}

impl Decodable for BlockTemplate {
    type Err = DecodingError;

    fn decode(buffer: &mut Buffer) -> Result<Self, Self::Err> {
        let count = buffer.read_varint()?;
        let mut txdata = Vec::new();

        for _ in 0..count.as_u64() {
            txdata.push(Transaction::decode(buffer)?);
        }

        let version = buffer.read_u32()?;
        let time = Time::decode(buffer)?;
        let height = buffer.read_u32()?;
        let bits = buffer.read_u32()?;
        let target = buffer.read_
        buffer.write_u256(self.target);
        buffer.extend(self.median_time.encode());
        buffer.write_u32(self.flags);
        buffer.write_u8(self.coinbase_flags.len() as u8);
        buffer.write_str(&self.coinbase_flags);
        buffer.extend(self.address.encode());
        buffer.write_u32(self.sigop_limit);
        buffer.write_u32(self.weight_limit);
        buffer.write_u32(self.opens);
        buffer.write_u32(self.updates);
        buffer.write_u32(self.renewals);
        buffer.write_u32(self.interval);
        buffer.write_u32(self.fees);
        buffer.extend(self.tree.encode());
        buffer.write_hash(self.previous_header_hash);
        buffer.write_hash(self.tree_root);
        buffer.write_hash(self.filter_root);
        buffer.write_hash(self.reserved_root);
        buffer.extend(self.right);
        buffer.extend(self.left);

        buffer.write_u8(self.transactions.len() as u8);

        for tx in self.transactions.iter() {
            buffer.extend(tx.encode());
        }

        Ok(Block { header, txdata })
    }
}
