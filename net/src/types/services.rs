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

        const FULL_NODE = 0 | Services::NETWORK.bits;
        const REQUIRED_SERVICES = 0 | Services::NETWORK.bits;
        const LOCAL_SERVICES = 0 | Services::NETWORK.bits;
    }
}
