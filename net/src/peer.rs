use crate::net_address::NetAddress;
use std::net::SocketAddr;
use crate::packets::{Packet, VersionPacket, UnknownPacket};
use log::warn;
//TODO reimplement when types crate is available.
use crate::types::{IdentityKey, ProtocolVersion};
use crate::Result;
use crate::error::Error;
use brontide::{BrontideStream, BrontideStreamBuilder};
use chrono::{DateTime, Utc};
use extended_primitives::Buffer;
use handshake_protocol::encoding::Encodable;
use handshake_protocol::network::Network;
use handshake_types::difficulty::Difficulty;
use romio::TcpStream;
use std::sync::{Arc, RwLock};
use futures::lock::Mutex;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Direction {
    Outbound = 0,
    Inbound = 1,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum State {
    Connected,
    Banned,
    //TODO might not need disconnected
    Disconnected,
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
    // pub user_agent: String,
    pub version: ProtocolVersion,
    pub address: NetAddress,
    // pub local_address: NetAddress,
    pub direction: Direction,
    // pub live_info: Arc<RwLock<PeerLiveInfo>>,
}

//TODO do we really need to have a network here?
pub struct Peer {
    pub info: PeerInfo,
    pub brontide: BrontideStream<TcpStream>,
    pub network: Network,
    pub state: Arc<RwLock<State>>,
    //TODO this might need to be RwLock
    pub loader: RwLock<bool>,
    pub queue: Mutex<Vec<Packet>>,
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
    //TODO should be a custom key type. - not sure if we want to store this inside of the peer.
    pub async fn connect(addr: NetAddress, key: [u8; 32], network: Network) -> Result<Peer> {
        //I think this returns a result.
        //AWait this. as it returns a future.
        //Result should be returned
        let socket = TcpStream::connect(&addr.address).await.unwrap();
        // let socket = TcpStream::connect(&"173.255.209.126:13038".parse().unwrap())
        //     .await
        //     .unwrap();

        let mut stream = BrontideStreamBuilder::new(socket, key)
            .connector(addr.key.as_array())
            .build();

        stream.start().await?;

        let peer_info = PeerInfo {
            address: addr,
            direction: Direction::Outbound,
            version: ProtocolVersion::from(0),
        };

        //TODO remove all non-future aware locks here
        let state = Arc::new(RwLock::new(State::Connected));

        Ok(Peer {
            info: peer_info,
            brontide: stream,
            loader: RwLock::new(false),
            network,
            state,
            queue: Mutex::new(Vec::new()),
        })
    }

    //Accept an incoming connection.
     // pub fn accept() {}

    //Handle all incoming messages, and put them into a message queue.
    pub async fn handle_messages(&mut self) -> Result<()> {

        //Need to check if this peer no longer exists on each loop, since otherwise we'll never
        //drop this.
        loop {
            let msg = self.next_message().await?;
            //For debugging.
            // dbg!(&msg);

            //Get lock on message queue
            // let mut queue = self.queue.lock().await;

            // queue.push(msg);
            //
            //TODO ugly, don't want to clone the message here, but can figure this out later.
            if let Packet::Version(version) = msg.clone() {
                self.handle_version(version).await?;
                //TODO actually don't make this an if else, we need to check version is received
                //BEFORE we even process a verack.
            } else if let Packet::Verack = msg {
                self.handle_verack().await?;
            };

            //Check for Verack here, if none, don't proceed -> Build a test to check for p2p
            //leakage. Aka if we are sending or processing other messages without getting a verack
            //and version.



            match msg {
                Packet::Unknown(unknown) => {
                    self.handle_unknown(unknown)
                },
                _ => unreachable!()
            };
        }

        Ok(())
    }

    pub fn handle_unknown(&self, msg: UnknownPacket) {
        warn!("Unknown packet: {} ({}).", msg._type as u32, self.hostname());
    }

    pub async fn handle_verack(&self) -> Result<()> {

        //TODO
        dbg!("Received Verack Packet!");
        Ok(())

    }

    pub async fn handle_version(&mut self, msg: VersionPacket) -> Result<()> {

        //if self.info.version.is_some() {
        //    //TODO destroy the peer. -> I Think, not sure if this is a destroyable offence.
        //    warn!("Peer sent a duplicate version.");
        //}
        dbg!("Received Version Packet");

        //Send back our own version.
        self.send_version().await?;

        //Send Verack
        self.send_verack().await?;

        Ok(())
    }
    // if (this.version !== -1)
    //   throw new Error('Peer sent a duplicate version.');

    // this.version = packet.version;
    // this.services = packet.services;
    // this.height = packet.height;
    // this.agent = packet.agent;
    // this.noRelay = packet.noRelay;
    // this.local = packet.remote;
    // // set the peer's key on their local address
    // this.local.setKey(this.address.getKey());

    // if (!this.network.selfConnect) {
    //   if (this.options.hasNonce(packet.nonce))
    //     throw new Error('We connected to ourself. Oops.');
    // }

    // if (this.version < common.MIN_VERSION)
    //   throw new Error('Peer does not support required protocol version.');

    // if (this.outbound) {
    //   if (!(this.services & services.NETWORK))
    //     throw new Error('Peer does not support network services.');

    //   if (this.options.spv) {
    //     if (!(this.services & services.BLOOM))
    //       throw new Error('Peer does not support BIP37.');
    //   }
    // }

    // this.send(new packets.VerackPacket());
    // this.logger.info(
    //   'Received version (%s): version=%d height=%d services=%s agent=%s',
    //   peer.hostname(),
    //   packet.version,
    //   packet.height,
    //   packet.services.toString(2),
    //   packet.agent);

    // this.network.time.add(peer.hostname(), packet.time);
    // this.nonces.remove(peer.hostname());

    // if (!peer.outbound && packet.remote.isRoutable())
    //   this.hosts.markLocal(packet.remote);




    pub async fn init_version(&mut self) -> Result<()> {
        self.send_version().await?;

        //Put timeout in next message here.
        let ack = self.next_message().await?;

        // let ack2 = self.brontide.next_message().await?;
        let ack2 = self.next_message().await?;

        dbg!(ack);

        dbg!(ack2);

        //await this has to occur in a timeout.
        //So we basically await the exact message size of ack, and then
        // self.receive_ack(

        Ok(())
    }

    //Wrapper around brontide's next message
    pub async fn next_message(&mut self) -> Result<Packet> {
        //Grab the next message from brontide
        //Attempt to parse it into a packet.
        //If it does parse into a packet, then return that packet,
        //otherwise error out.
        //
        let raw_packet = self.brontide.next_message().await?;

        let packet = Packet::decode(Buffer::from(raw_packet))?;

        Ok(packet)
    }

    pub async fn send_version(&mut self) -> Result<()> {
        //Need to pass in height dynamically. TODO
        //Also need to pass in no_relay dynamically TODO
        let packet = Packet::Version(VersionPacket::new(self.info.address, 0, false));
        //Each packet might have a different timeout requirement -> We should probably set this in
        //the packet struct itself.
        //
        self.brontide
            .write(packet.frame(self.network).to_vec())
            .await?;

        Ok(())
    }

    pub async fn send_verack(&mut self) -> Result<()> {
        let packet = Packet::Verack;

        self.brontide.write(packet.frame(self.network).to_vec()).await?;

        Ok(())

    }

    pub fn is_connected(&self) -> bool {
            let state = match self.state.read() {
                Ok(state) => state,
                Err(_) => return false,
            };

        State::Connected == *state
    }

    pub fn is_outbound(&self) -> bool {
        Direction::Outbound == self.info.direction
    }

    pub fn set_loader(&self, load: bool) -> Result<()> {
        let mut loader = match self.loader.write() {
            Ok(loader) => loader,
            Err(_) => return Err(Error::LockError),
        };

        *loader = load;

        Ok(())

    }

    pub fn hostname(&self) -> SocketAddr {
        self.info.address.get_socket_addr()
    }

    // pub async fn receive_version(&mut self, packet: Packet::Version) -> Result<()> {
    //     if self.info.version

    // }
        // async handleVersion(packet) {
    // if (this.version !== -1)
      // throw new Error('Peer sent a duplicate version.');

    // this.version = packet.version;
    // this.services = packet.services;
    // this.height = packet.height;
    // this.agent = packet.agent;
    // this.noRelay = packet.noRelay;
    // this.local = packet.remote;
    // // set the peer's key on their local address
    // this.local.setKey(this.address.getKey());

    // if (!this.network.selfConnect) {
      // if (this.options.hasNonce(packet.nonce))
        // throw new Error('We connected to ourself. Oops.');
    // }

    // if (this.version < common.MIN_VERSION)
      // throw new Error('Peer does not support required protocol version.');

    // if (this.outbound) {
      // if (!(this.services & services.NETWORK))
        // throw new Error('Peer does not support network services.');

      // if (this.options.spv) {
        // if (!(this.services & services.BLOOM))
        //   throw new Error('Peer does not support BIP37.');
      // }
    // }

    // this.send(new packets.VerackPacket());
  // }


    //TODO function that writes to the stream and takes a generic packet.
    //We set lastsend in this function.
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::executor;
    use std::net::{IpAddr, Ipv4Addr, SocketAddr};
    use std::str::FromStr;
    use crate::seeds;

    #[test]
    fn test_peer_connect() {
        executor::block_on(async {

        let local_key = [1; 32];

        let seeds = seeds::testnet_seed_nodes();
        // let peer_address: NetAddress = seeds[3].parse().unwrap();
        let peer_address: NetAddress = "ak2hy7feae2o5pfzsdzw3cxkxsu3lxypykcl6iphnup4adf2ply6a@138.68.61.31:13038".parse().unwrap();

        dbg!(&peer_address);

        let mut peer = Peer::connect(peer_address, local_key, Network::Testnet).await.unwrap();

        // peer.init_version().await.unwrap();
        peer.handle_messages().await;

        ()
    })
    }

}
