#[derive(Copy, Clone, Debug)]
pub struct RawIP([u8; 16]);

//TODO figure out how best to expose this/if we want to at all
//If we can not expose this, I think that's the best.
pub enum IPNetwork {
    None = 0,
    Inet4 = 1,
    Inet6 = 2,
    Onion = 3,
    Teredo = 4
}

impl RawIP {

    pub fn is_null(&self) -> bool {
        if self.is_IPv4() {
            // 0.0.0.0
            return self.0[12] == 0 && self.0[13] == 0 && self.0[14] == 0 && self.0[15] == 0;
        }
        self.0 == [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]
    }

    pub fn is_broadcast(&self) -> bool {
        if !self.is_IPv4() {
            return false;
        }

        self.0[12] == 255 && self.0[13] == 255 && self.0[14] == 255 && self.0[15] == 255
    }

    pub fn is_IPv4(&self) -> bool {
        (self.0[0] == 0
            && self.0[1] == 0
            && self.0[2] == 0
            && self.0[3] == 0
            && self.0[4] == 0
            && self.0[5] == 0
            && self.0[6] == 0
            && self.0[7] == 0
            && self.0[8] == 0
            && self.0[9] == 0
            && self.0[10] == 0xff
            && self.0[11] == 0xff)
    }

    ///Tests whether the IP is RFC1918 (Private internet) https://tools.ietf.org/html/rfc1918
    pub fn is_RFC1918(&self) -> bool {
        if !self.is_IPv4() {
            return false;
        }

        // 10.0.0.0/8
        if self.0[12] == 10 {
            return true;
        }

        // 192.168.0.0/16
        if self.0[12] == 192 && self.0[13] == 168 {
            return true;
        }

        // 172.16.0.0/12
        if self.0[12] == 172 && self.0[13] >= 16 && self.0[13] <= 31 {
            return true;
        }

        false
    }

    /// Tests whether the IP is RFC2544 (Filter Addresses) https://tools.ietf.org/html/rfc2544
    pub fn is_RFC2544(&self) -> bool {
        if !self.is_IPv4() {
            return false;
        }

        // 198.18.1.2 -> 198.19.65.2
        if self.0[12] == 198 && (self.0[13] == 18 || self.0[13] == 19) {
            return true;
        }

        false
    }

    /// Tests whether the IP is RFC3927 (Link-Local Addresses) https://tools.ietf.org/html/rfc3927
    pub fn is_RFC3927(&self) -> bool {
        if !self.is_IPv4() {
            return false;
        }

        // 169.254.0.0/16
        if self.0[12] == 169 && self.0[13] == 254 {
            return true;
        }

        false
    }

    /// Tests whether the IP is RFC6598 (Shared Address Space) https://tools.ietf.org/html/rfc6598
    pub fn is_RFC6598(&self) -> bool {
        if !self.is_IPv4() {
            return false;
        }

        // 100.64.0.0/10
        if self.0[12] == 100 && (self.0[13] >= 64 && self.0[13] <= 127) {
            return true;
        }

        false
    }

    /// Tests whether the IP is RFC5737 (Documentation Addresses) https://tools.ietf.org/html/rfc5737
    pub fn is_RFC5737(&self) -> bool {
        if !self.is_IPv4() {
            return false;
        }

        // 192.0.2.0/24
        if self.0[12] == 192 && self.0[13] == 0 && self.0[14] == 2 {
            return true;
        }

        // 198.51.100.0/24
        if self.0[12] == 198 && self.0[13] == 51 && self.0[14] == 100 {
            return true;
        }

        // 203.0.113.0/24
        if self.0[12] == 203 && self.0[13] == 0 && self.0[14] == 113 {
            return true;
        }

        false
    }

    /// Tests whether the IP is RFC3849 (IPv6 Documentation Addresses) https://tools.ietf.org/html/rfc3849
    pub fn is_RFC3849(&self) -> bool {
        // 2001:0DB8::/32
        self.0[0] == 0x20 && self.0[1] == 0x01 && self.0[2] == 0x0d && self.0[3] == 0xb8
    }

    /// Tests whether the IP is RFC3964 (IPv6 6to4 Tunnelling Addresses) https://tools.ietf.org/html/rfc3964
    pub fn is_RFC3964(&self) -> bool {
        // 2002::/16
        self.0[0] == 0x20 && self.0[1] == 0x02
    }

    /// Tests whether the IP is RFC6052 (IPv6 Prefix for IPv4 Embedded Addresses) https://tools.ietf.org/html/rfc6052
    pub fn is_RFC6052(&self) -> bool {
        // 64:FF9B::/96
        has_prefix(vec![0, 0x64, 0xFF, 0x9B, 0, 0, 0, 0, 0, 0, 0, 0], self.0)
    }

    /// Tests whether the IP is RFC4380 (IPv6 Teredo Tunnelling Addresses) https://www.ietf.org/rfc/rfc4380.txt
    pub fn is_RFC4380(&self) -> bool {
        // 2001:0000:/32
        self.0[0] == 0x20 && self.0[1] == 0x01 && self.0[2] == 0x00 && self.0[3] == 0x00
    }

    /// Tests whether the IP is RFC4862 (IPv6 Link-Local Addresses) https://tools.ietf.org/html/rfc4862
    pub fn is_RFC4862(&self) -> bool {
        //FE80::/64
        has_prefix(vec![0xFE, 0x80, 0, 0, 0, 0, 0, 0], self.0)
    }

    /// Tests whether the IP is RFC4193 (IPv6 Unique Local Addresses) https://tools.ietf.org/html/rfc4193
    pub fn is_RFC4193(&self) -> bool {
        // FC00::/7
        self.0[0] & 0xFE) == 0xFC
    }

    /// Tests whether the IP is RFC6145 (IPv6 IPv4-Translated Address) https://tools.ietf.org/html/rfc6145
    pub fn is_RFC6145(&self) -> bool {
        // ::FFFF:0:0:0/96
        has_prefix(vec![0,0,0,0,0,0,0,0,0xFF,0xFF,0,0], self.0)
    }

    /// Tests whether the IP is RFC4843 (deprecated) (IPv6 Prefix for ORCHID) https://tools.ietf.org/html/rfc4843
    pub fn is_RFC4843(&self) -> bool {
        // 2001:10::/28
        self.0[0] == 0x20 && self.0[1] == 0x01 && self.0[2] == 0x00 && (self.0[3] & 0xF0) == 0x10
    }

    /// Tests whether the IP is RFC7343 (IPv6 Prefix for ORCHID v2) https://tools.ietf.org/html/rfc7343
    pub fn is_RFC7343(&self) -> bool {
        // 2001:20::/28
        self.0[0] == 0x20 && self.0[1] == 0x01 && self.0[2] == 0x00 && (self.0[3] & 0xF0) == 0x20
    }

    /// Test whether the IP is local
    pub fn is_local(&self) -> bool {

        // IPv4 loopback (127.0.0.0/8 or 0.0.0.0/8)
        if self.is_IPv4() && (self.0[12] == 127 || self.0[13] == 0) {
            return true;
        }

          // IPv6 loopback (::1/128)
        if self.0 == [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1] {
            return true;
        }

        false;
    }

    //TODO
    pub fn is_onion(&self) -> bool {
        unimplement!()
    }

    /// Test whether the IP is valid
    pub fn is_valid(&self) -> bool {
        //Only triggers in Bitcoin Versions 0.2.9 - Not needed for Handshake, but this code will
        //likely make it into Bitcoin code so we should keep.
        // if has_prefix(vec![00, 00, 00, 00, 00, 00, 00, ff, ff], self.0) {
        //     return false;
        // }

        if self.is_null() {
            return false;
        }

        if self.is_broadcast() {
            return false;
        }

        if self.is_RFC3849() {
            return false;
        }

        true
    }

    pub fn is_routable(&self) -> bool {
        return self.is_valid() && !(self.is_RFC1918()
            || self.is_RFC2544()
            || self.is_RFC3927()
            || self.is_RFC4862()
            || self.is_RFC6598()
            || self.is_RFC5737()
            || (self.is_RFC4193() && !self.is_Onion())
            || self.is_RFC4843()
            || self.is_RFC7343()
            || self.is_Local()
    }

    //TODO add unroutable?
    pub fn get_network(&self) -> IPNetwork {
        if self.is_IPv4() {
            return IPNetwork::Inet4;
        }

        if self.is_RFC4380() {
            return IPNetwork::Teredo;
        }

        if self.is_Onion() {
            return IPNetwork::Onion;
        }

        IPNetwork::Inet6
    }

    // TODO
    pub fn get_reachability(&self) {
        unimplemented!()
    }

}

//Helper function
fn has_prefix(prefix: Vec<u8>, ip: [u8; 16]) -> bool {
    if prefix.len() > 16 {
        return false;
    }

    for i in 0..prefix.len() {
        if prefix[i] != ip[i] {
            return false;
        }
    }

    true
}
//TODO implement return SocketAddr
//TODO implement from string
//TODO implement to string
//TODO implement Eqs, Partial Eq
//TODO implement deref.
//TODO implement custom Debug.
//TODO implement froms for SocketAddr
