use handshake_types::Hash;

//TODO should we impl Odr?
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
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

    pub fn is_null(&self) -> bool {
        *self == Outpoint::null()
    }
}
