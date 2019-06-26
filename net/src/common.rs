/// Default protocol version
pub const PROTOCOL_VERSION: u32 = 1;

/// Minimum protocol version that we'll talk to
pub const MIN_PROTOCOL_VERSION: u32 = 1;

//TODO change this now since net version might be different than Rust
/// User agent passed along in messages
pub const USER_AGENT: &str = concat!("RSD:", env!("CARGO_PKG_VERSION"));

///Maximum message size that can be sent ~8mb
pub const MAX_MESSAGE_SIZE: u32 = 8_000_000;

///Amount of time to ban misbehaving peers
pub const BAN_TIME: u32 = 86_400;

///Ban score threshold before ban is placed in effect
pub const BAN_SCORE: u32 = 100;

///Maximum inv/getdata size
pub const MAX_INV: u32 = 50_000;

///Maximum number of requests
pub const MAX_REQUEST: u32 = 5_000;

///Maximum number of block requests
pub const MAX_BLOCK_REQUEST: u32 = 50_000 + 1_000;

///Maximum number of transaction requests
pub const MAX_TX_REQUEST: u32 = 10_000;

///Maximum number of claim requests
pub const MAX_CLAIM_REQUEST: u32 = 1_000;
