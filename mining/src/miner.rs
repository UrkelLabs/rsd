use handshake_primitives::Address;

pub struct Miner {
    pub address: Address,
    //I think option makes the most sense here so we don't force users to have flags.
    pub coinbase_flags: Option<String>,
}
