use crate::{Input, Output};
use extended_primitives::Hash;

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Transaction {
    /// The protocol version, is currently expected to be 1 or 2 (BIP 68).
    pub version: u32,
    /// Block number before which this transaction is valid, or 0 for
    /// valid immediately.
    pub lock_time: u32,
    //TODO inputs and outputs.
    //
    pub inputs: Vec<Input>,
    pub outputs: Vec<Output>,
}

impl Transaction {
    //TODO implement - see hsd primitives
    //we should keep similar behavior with hash and witness hash
    pub fn hash(&self) -> Hash {
        Default::default()
    }
}
