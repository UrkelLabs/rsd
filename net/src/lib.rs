#![feature(async_await)]

pub mod common;
pub mod error;
pub mod net_address;
pub mod packets;
pub mod peer;
pub mod peer_list;
pub mod peer_store;
pub mod pool;
pub mod seeds;
pub mod types;

pub type Result<T> = std::result::Result<T, error::Error>;

pub use net_address::NetAddress;

//TODO export all types that we need are needed publically.
//Also, ensure that the properties and functions in those types are public if they need to be
//public.
//
//Other things should be marked as pub in crate only.
