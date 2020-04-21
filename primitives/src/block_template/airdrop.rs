use crate::address::Payload;
use crate::{Address, Input, Output, Transaction};
use encodings::FromHex;
use extended_primitives::{Buffer, Hash, Uint256};

#[cfg(feature = "json")]
use crate::block_template::json::AirdropEntry;

pub struct BlockAirdrop {
    pub blob: Buffer,
    pub position: usize,
    pub address: Address,
    pub value: u64,
    pub fee: u64,
    pub rate: f64,
    pub weak: bool,
}

impl BlockAirdrop {
    #[cfg(feature = "json")]
    pub fn from_entry(entry: AirdropEntry) -> Self {
        let payload = Payload::from_hash(Buffer::from_hex(entry.address).unwrap()).unwrap();
        BlockAirdrop {
            blob: Buffer::from_hex(entry.data).unwrap(),
            position: entry.position,
            address: Address {
                version: entry.version,
                hash: payload,
            },
            value: entry.value,
            fee: entry.fee,
            rate: entry.rate,
            weak: entry.weak,
        }
    }
}
