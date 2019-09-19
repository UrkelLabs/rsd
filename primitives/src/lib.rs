pub mod address;
pub mod block;
pub mod block_template;
pub mod covenant;
pub mod headers;
pub mod input;
pub mod inventory;
pub mod outpoint;
pub mod output;
pub mod transaction;

pub use crate::address::Address;
pub use crate::block::Block;
pub use crate::block_template::BlockTemplate;
pub use crate::covenant::Covenant;
pub use crate::headers::BlockHeader;
pub use crate::input::Input;
pub use crate::inventory::Inventory;
pub use crate::outpoint::Outpoint;
pub use crate::output::Output;
pub use crate::transaction::Transaction;

//@todo we are starting to get a few too many primitives in here, so I think this calls for some
//structuring.
//
//My initial thoughts:
//block
//  -> Block
//  -> Block Header
//  -> Block Template
//transaction or tx
//  -> Covenant
//  -> Input
//  -> Outpoint
//  -> Output
//  -> Transaction
//airdrop
//  -> AirdropClaim
//  -> AirdropProof
//
//  @todo Covenant should be it's own folder and have a file for each type.
