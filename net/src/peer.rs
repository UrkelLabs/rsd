use crate::net_address::NetAddress;
use crate::packets::{
    AddrPacket, GetBlocksPacket, InvPacket, Packet, PingPacket, PongPacket, UnknownPacket,
    VersionPacket,
};
use handshake_types::Time;
use log::warn;
use std::net::SocketAddr;
//TODO reimplement when types crate is available.
use crate::error::Error;
use crate::types::{IdentityKey, Nonce, ProtocolVersion, Services};
use crate::Result;
use brontide::{BrontideStream, BrontideStreamBuilder};
use chrono::{DateTime, Utc};
use extended_primitives::Buffer;
use futures::channel::mpsc::UnboundedSender;
use futures::lock::Mutex;
use futures::sink::SinkExt;
use handshake_protocol::encoding::Encodable;
use handshake_protocol::network::Network;
use handshake_types::difficulty::Difficulty;
use romio::TcpStream;
use std::sync::{Arc, RwLock};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Direction {
    Outbound = 0,
    Inbound = 1,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum State {
    Connected,
    Initial,
    Banned,
    //TODO might not need disconnected
    Disconnected,
}

//Defaults
#[derive(Clone, Debug)]
pub struct PeerLiveInfo {
    // pub total_difficulty: Difficulty,
    pub height: u32,
    pub last_seen: Time,
    pub first_seen: Time,
    pub last_send: Time,
    pub last_receive: Time,
}

//Unchanging information about a peer.
#[derive(Clone, Debug)]
pub struct PeerInfo {
    pub user_agent: String,
    pub version: Option<ProtocolVersion>,
    pub address: NetAddress,
    pub services: Services,
    pub no_relay: bool,
    pub direction: Direction,
}

//Information about a peers ping/pong responses.
#[derive(Clone, Debug)]
pub struct PingStats {
    challenge: Option<Nonce>,
    last_pong: Time,
    last_ping: Time,
    min_ping: Time,
}

//TODO do we really need to have a network here?
#[derive(Debug)]
pub struct Peer {
    //ARC
    pub info: PeerInfo,
    //ARC
    pub live_info: Mutex<PeerLiveInfo>,
    //We might want to break this into 2, one for writing and one for reading.
    //would prevent locks on both, and increase the speed here. TODO
    //The likely way to do this would actually be in the brontide package, where we just expose a
    //mspc or a oneshot channel to one async function that loops through all messages and then
    //reads and or writes to them.
    //ARC
    pub brontide: Mutex<BrontideStream<TcpStream>>,
    //ARC
    pub network: Network,
    //Possibly combine state and live info into the same lock.
    pub state: Arc<Mutex<State>>,
    //TODO this might need to be RwLock
    pub loader: RwLock<bool>,
    //ARC
    pub tx: Mutex<UnboundedSender<Packet>>,
    pub ping_stats: Mutex<PingStats>,
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
    //Connect to a new peer.
    //TODO should be a custom key type. - not sure if we want to store this inside of the peer.
    pub async fn connect(
        addr: NetAddress,
        key: [u8; 32],
        network: Network,
        tx: UnboundedSender<Packet>,
    ) -> Result<Peer> {
        //TODO catch error, don't unwrap.
        let socket = TcpStream::connect(&addr.address).await.unwrap();

        let mut stream = BrontideStreamBuilder::new(socket, key)
            .connector(addr.key.as_array())
            .build();

        stream.start().await?;

        //TODO split the stream to readers and writers
        //TODO maybe should make these their own structs. BrontideReader, BrontideWriter
        //let (brx, btx) = stream.split();

        let info = PeerInfo {
            address: addr,
            user_agent: "".to_owned(),
            //TODO not sure what to default this to.
            no_relay: false,
            direction: Direction::Outbound,
            version: None,
            services: Services::empty(),
        };

        let live_info = PeerLiveInfo {
            height: 0,
            last_seen: Time::new(),
            first_seen: Time::new(),
            last_send: Time::new(),
            last_receive: Time::new(),
        };

        let ping_stats = PingStats {
            challenge: None,
            last_ping: Time::new(),
            last_pong: Time::new(),
            min_ping: Time::new(),
        };

        let state = Arc::new(Mutex::new(State::Initial));

        Ok(Peer {
            info,
            live_info: Mutex::new(live_info),
            brontide: Mutex::new(stream),
            loader: RwLock::new(false),
            network,
            state,
            tx: Mutex::new(tx),
            ping_stats: Mutex::new(ping_stats),
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

            if let Packet::Version(version) = &msg {
                self.handle_version(version).await?;
            }

            //If we have not received a version, then continue, and add to the peers ban score.
            if self.info.version.is_none() {
                // self.increase_ban(1);
                continue;
            }

            if msg == Packet::Verack {
                self.handle_verack().await?;
            }

            //Get state lock
            let state = self.state.lock().await;

            //TODO maybe just check for banned/disconnected here as well. -> Could do a match
            //statement. Although, it probably should be above the next message stuff.
            //If we have not received a verack, then continue and add a ban score.
            if *state != State::Connected {
                // self.increase_ban(1);
                continue;
            }

            match &msg {
                //TODO ping, pong, sendheaders, filterload, filteradd, filterclear, feefilter,
                //sendcompact
                Packet::Ping(ping) => self.handle_ping(ping).await?,
                Packet::Pong(pong) => self.handle_pong(pong).await?,
                //Remaining packets, do nothing. They are sent to the pool.
                _ => {}
            };

            //Acquire tx lock.
            let mut tx = self.tx.lock().await;

            //TODO need to implement the error here
            tx.send(msg).await;
        }

        Ok(())
    }

    pub async fn handle_version(&mut self, msg: &VersionPacket) -> Result<()> {
        if self.info.version.is_some() {
            warn!("Peer sent a duplcation version.");
            // Increase ban by 1.
            // self.increase_ban(1);
        }

        //Do all non-chaning info here.
        self.info.version = Some(msg.version);
        self.info.services = msg.services;
        self.info.user_agent = msg.agent.clone();
        self.info.no_relay = msg.no_relay;

        //Acquire lock, and change live info.
        let mut live_info = self.live_info.lock().await;

        //TODO do we set interaction stuff here?
        live_info.height = msg.height;

        dbg!(&self);

        //Send back our own version.
        self.send_version().await?;

        //Send Verack
        self.send_verack().await?;

        Ok(())
    }

    pub async fn handle_verack(&self) -> Result<()> {
        //TODO see if currentlyConnected is important or not.
        //if self.info.direction == Direction::Outbound {
        //    //Get state lock.
        //    let mut state = self.state.lock().await;

        //    *state = State::Connected;
        //    // info!("New outbound peer connected: version: {}, blocks: {}, peer: {}", self.info.version, self.info.address);
        //}

        //    //Get state lock.
        let mut state = self.state.lock().await;

        //Mark the node as connected.
        *state = State::Connected;

        Ok(())
    }

    pub async fn handle_ping(&self, msg: &PingPacket) -> Result<()> {
        //Assume the packets always have nonce. Write a test to ensure that this is the case TODO
        //The test should try to send a ping that does not have a nonce, and we should handle it
        //accordingly.
        let packet = PongPacket::new(msg.nonce);

        self.send(Packet::Pong(packet)).await?;

        Ok(())
    }

    pub async fn handle_pong(&self, msg: &PongPacket) -> Result<()> {
        let nonce = msg.nonce;
        let now = Time::now();

        //Acquire ping stats lock
        let stats = self.ping_stats.lock().await;

        if let Some(challenge_nonce) = stats.challenge {
            if nonce != challenge_nonce {
                if nonce == 0 {
                    // info!("Peer sent a zero nonce {}", self.info.address);
                    stats.challenge = None;
                    return Ok(());
                }
                // info!("Peer sent the wrong nonce {}.", self.info.address);
                return Ok(());
            }

            if now >= stats.last_pong {
                stats.last_pong = now;
                if stats.min_ping == 0 {
                    stats.min_ping = now - stats.last_ping;
                }
                stats.min_ping = std::cmp::min(stats.min_ping, now - stats.last_ping);
            } else {
                // info!("Timing mismatch {}", self.info.address);
            }
        } else {
            // info!("Peer sent an unsolicited pong {}", self.info.address);
        }

        stats.challenge = None;

        Ok(())
    }

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
    //TODO this needs to be tested as I think it might be holding the lock not allowing sending to
    //occur.
    pub async fn next_message(&self) -> Result<Packet> {
        //Grab brontide lock -> TODO make sure this is working cleanly.
        //We might be locking here until next message is received, in which case we want to drop
        //the lock.
        let mut brontide = self.brontide.lock().await;
        let raw_packet = brontide.next_message().await?;

        let packet = Packet::decode(Buffer::from(raw_packet))?;

        Ok(packet)
    }

    //Change brontide to be a mutex, then this isn't mut
    pub async fn send(&self, packet: Packet) -> Result<()> {
        //Acquire Brontide Lock TODO should just be writing lock.
        let mut brontide = self.brontide.lock().await;

        brontide.write(packet.frame(self.network).to_vec()).await?;

        Ok(())
    }

    pub async fn send_version(&self) -> Result<()> {
        //Need to pass in height dynamically. TODO
        //Also need to pass in no_relay dynamically TODO
        let packet = Packet::Version(VersionPacket::new(self.info.address, 0, false));
        //Each packet might have a different timeout requirement -> We should probably set this in
        //the packet struct itself.
        self.send(packet).await?;

        Ok(())
    }

    pub async fn send_verack(&self) -> Result<()> {
        let packet = Packet::Verack;

        self.send(packet).await?;

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

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use crate::seeds;
//     use futures::executor;
//     use std::net::{IpAddr, Ipv4Addr, SocketAddr};
//     use std::str::FromStr;

//     // #[test]
//     // fn test_peer_connect() {
//     //     executor::block_on(async {

//     //     let local_key = [1; 32];

//     //     let seeds = seeds::testnet_seed_nodes();
//     //     // let peer_address: NetAddress = seeds[3].parse().unwrap();
//     //     let peer_address: NetAddress = "ak2hy7feae2o5pfzsdzw3cxkxsu3lxypykcl6iphnup4adf2ply6a@138.68.61.31:13038".parse().unwrap();

//     //     dbg!(&peer_address);

//     //     let mut peer = Peer::connect(peer_address, local_key, Network::Testnet).await.unwrap();

//     //     // peer.init_version().await.unwrap();
//     //     peer.handle_messages().await;

//     //     ()
//     // })
//     // }

// }
