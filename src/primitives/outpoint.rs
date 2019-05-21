use crate::Hash;

pub struct Outpoint {
    txid: Hash,
    index: u32,
}

impl Outpoint {
    ///Returns a null Outpoint for use in coinbase transactions.
    pub fn null() -> Outpoint {
        Outpoint {
            txid: Default::default(),
            index: u32::max_value(),
        }
    }
}
