use crate::primitives::headers::BlockHeader;
use crate::primitives::transaction::Transaction;

/// A Handshake block, which is a collection of transactions with an attached
/// proof of work.
// #[derive(PartialEq, Eq, Clone, Debug)]
pub struct Block {
    /// The block header
    pub header: BlockHeader,
    /// List of transactions contained in the block
    pub txdata: Vec<Transaction>
}
