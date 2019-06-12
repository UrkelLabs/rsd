use crate::net::packets::VersionPacket;
use crate::net::types::{IdentityKey, PeerAddr, ProtocolVersion};
use crate::types::difficulty::Difficulty;
use brontide::BrontideStream;
use chrono::{DateTime, Utc};
use romio::TcpStream;
use std::sync::{Arc, RwLock};

#[derive(Clone, Debug)]
pub enum Direction {
    Outbound = 0,
    Inbound = 1,
}

#[derive(Clone, Debug)]
pub struct PeerLiveInfo {
    pub total_difficulty: Difficulty,
    pub height: u64,
    pub last_seen: DateTime<Utc>,
    //TODO see if still necessary.
    pub stuck_detector: DateTime<Utc>,
    pub first_seen: DateTime<Utc>,
    pub last_send: DateTime<Utc>,
    pub last_receive: DateTime<Utc>,
}

#[derive(Clone, Debug)]
pub struct PeerInfo {
    pub user_agent: String,
    pub version: ProtocolVersion,
    pub address: PeerAddr,
    pub local_address: PeerAddr,
    pub direction: Direction,
    pub live_info: Arc<RwLock<PeerLiveInfo>>,
}

pub struct Peer {
    pub info: PeerInfo,
    pub key: IdentityKey,
    pub brontide: BrontideStream<TcpStream>,
    // state: Arc<RwLock<State>>,
    // // set of all hashes known to this peer (so no need to send)
    // tracking_adapter: TrackingAdapter,
    // tracker: Arc<conn::Tracker>,
    // send_handle: Mutex<conn::ConnHandle>,
    // // we need a special lock for stop operation, can't reuse handle mutex for that
    // // because it may be locked by different reasons, so we should wait for that, close
    // // mutex can be taken only during shutdown, it happens once
    // stop_handle: Mutex<conn::StopHandle>
}

impl Peer {
    //Internal function to create peers, NOT public
    fn new() {}

    //Connect to a new peer.
    //TODO return a result
    //TODO should be a custom key type. - not sure if we want to store this inside of the peer.
    pub async fn connect(addr: PeerAddr, key: [u8; 32]) {
        //I think this returns a result.
        //AWait this. as it returns a future.
        //Result should be returned
        // let socket = TcpStream::connect(&addr.address).await.expect("socket should connect");
        let socket = TcpStream::connect(&"173.255.209.126:13038".parse().unwrap())
            .await
            .unwrap();

        let mut stream = BrontideStream::connect(socket, key, addr.key.as_array()).await;
    }

    //Accept an incoming connection.
    pub fn accept() {}

    pub fn send_version(&self) {
        //Need to pass in height dynamically. TODO
        //Also need to pass in no_relay dynamically TODO
        let packet = VersionPacket::new(self.info.address, 0, false);
        //Each packet might have a different timeout requirement -> We should probably set this in
        //the packet struct itself.

        //We need to encode this packet, and then frame it.
        //
        //After that is done, we send it through the brontide.
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::executor;
    use std::net::{IpAddr, Ipv4Addr, SocketAddr};
    use std::str::FromStr;

    // #[test]
    //fn test_peer_connect() {
    //    executor::block_on(async {
    //    //TODO get the port from the network.
    //    let address = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(172, 104, 214, 189)), 13038);

    //    let key =
    //        IdentityKey::from_str("ajdzrpoxsusaw4ixq4ttibxxsuh5fkkduc5qszyboidif2z25i362").unwrap();

    //    let local_key = [1; 32];

    //    let peer_address = PeerAddr::new(address, key);
    //    let peer = Peer::connect(peer_address, local_key).await;

    //    ()
    //})
    //}

}

// this.parser = new Parser(this.network);
// this.framer = new Framer(this.network);

// this.id = -1;
// this.socket = null;
// this.brontide = new BrontideStream();
// this.opened = false;
// this.loader = false;
// this.name = null;
// this.connected = false;
// this.destroyed = false;
// this.ack = false;
// this.handshake = false;
// this.time = 0;
// this.drainSize = 0;
// this.drainQueue = [];
// this.banScore = 0;
// this.invQueue = [];
// this.onPacket = null;

// this.next = null;
// this.prev = null;

// this.agent = null;
// this.noRelay = false;
// this.preferHeaders = false;
// this.hashContinue = consensus.ZERO_HASH;
// this.spvFilter = null;
// this.feeRate = -1;
// this.compactMode = -1;
// this.merkleBlock = null;
// this.merkleTime = -1;
// this.merkleMatches = 0;
// this.merkleMap = null;
// this.syncing = false;
// this.sentAddr = false;
// this.sentGetAddr = false;
// this.challenge = null;
// this.lastPong = -1;
// this.lastPing = -1;
// this.minPing = -1;
// this.blockTime = -1;

// this.bestHash = consensus.ZERO_HASH;
// this.bestHeight = -1;

// this.lastTip = consensus.ZERO_HASH;
// this.lastStop = consensus.ZERO_HASH;

// this.connectTimeout = null;
// this.pingTimer = null;
// this.invTimer = null;
// this.stallTimer = null;

// this.addrFilter = new RollingFilter(5000, 0.001);
// this.invFilter = new RollingFilter(50000, 0.000001);

// this.blockMap = new BufferMap();
// this.txMap = new BufferMap();
// this.claimMap = new BufferMap();
// this.airdropMap = new BufferMap();
// this.responseMap = new Map();
// this.compactBlocks = new BufferMap();
// this.nameMap = new BufferMap();
// this.totalProofs = 0;
