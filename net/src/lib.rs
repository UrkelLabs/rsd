#![feature(async_await)]

pub mod common;
pub mod error;
pub mod net_address;
pub mod packets;
pub mod peer;
pub mod pool;
pub mod types;

pub type Result<T> = std::result::Result<T, error::Error>;
