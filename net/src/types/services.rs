use crate::common::{BLOOM, FULL_NODE, NETWORK};
use crate::error;
use std::convert::TryFrom;

use bitflags::bitflags;

bitflags! {
    /// Options for what type of interaction a peer supports
    pub struct Services: u32 {
        /// We don't know (yet) what the peer can do.
        const UNKNOWN = 0b00000000;
        /// Service constant for Network capabilities (1 << 0)
        const NETWORK = 0b00000001;
        ///Service constant for Bloom Filter capabilities
        const BLOOM = 0b00000010;

        pub const FULL_NODE = 0 | Services::NETWORK.bits;
        pub const REQUIRED_SERVICES = 0 | Services::NETWORK.bits;
        pub const LOCAL_SERVICES = 0 | Services::NETWORK.bits;

        /// All nodes right now are "full nodes".
        /// Some nodes internally may maintain longer block histories (archival_mode)
        /// but we do not advertise this to other nodes.
        /// All nodes by default will accept lightweight "kernel first" tx broadcast.
        const FULL_NODE = Capabilities::HEADER_HIST.bits
            | Capabilities::TXHASHSET_HIST.bits
            | Capabilities::PEER_LIST.bits
            | Capabilities::TX_KERNEL_HASH.bits;
    }
}
