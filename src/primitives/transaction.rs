#[derive(Clone, PartialEq, Eq, Debug, Hash)]
pub struct Transaction {
    /// The protocol version, is currently expected to be 1 or 2 (BIP 68).
    /// TODO this might need to be a i32
    pub version: u32,
    /// Block number before which this transaction is valid, or 0 for
    /// valid immediately.
    pub lock_time: u32,
    //TODO inputs and outputs.
}
