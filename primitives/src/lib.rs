pub mod address;
pub mod block;
pub mod block_template;
pub mod covenants;
pub mod headers;
pub mod inventory;
pub mod transaction;

pub use crate::address::Address;
pub use crate::block::Block;
pub use crate::block_template::BlockTemplate;
pub use crate::covenants::Covenant;
pub use crate::headers::BlockHeader;
pub use crate::inventory::Inventory;
pub use crate::transaction::{Output, Input, Outpoint, Transaction};

//@todo we are starting to get a few too many primitives in here, so I think this calls for some
//structuring.
//
//My initial thoughts:
//block
//  -> Block
//  -> Block Header
//  -> Block Template
//transaction or tx
//  -> Input
//  -> Outpoint
//  -> Output
//  -> Transaction
//airdrop
//  -> AirdropClaim
//  -> AirdropProof
