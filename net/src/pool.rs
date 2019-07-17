use crate::peer::Peer;
use crate::peer_list::PeerList;
use handshake_types::Time;
use crate::Result;
// use futures::task::{Spawn, SpawnExt};
use futures_timer::Delay;
use futures::lock::Mutex;
use log::info;
use std::sync::Arc;
use std::time::Duration;
use crate::peer_store::PeerStore;
use crate::NetAddress;
use handshake_protocol::network::Network;

//TODO cleanup imports
// use crate::blockchain::chain::Chain;
// use crate::protocol::network::Network;

//Set defaults for these. Also probably put these in it's own file -> config.rs
//Should also probably be named something like p2p config or net config.
#[derive(Clone)]
pub struct PoolConfig {
    pub max_outbound: u32,
    pub max_inbound: u32,
}

//TODO possible name this p2p server.
//TODO need our max reachable connections.
#[derive(Clone)]
pub struct Pool {
    //TODO does this actually need to be an arc?
    peers: Arc<Mutex<PeerList>>,
    config: PoolConfig,
    connected: bool,
    store: Arc<PeerStore>,
    // connected_groups: Mutex<Vec<Vec<u8>>>
}

//Juliex doesn't impl Spawn or Spawn Ext -> Either this is my misunderstanding of how to use these
//traits, or it needs to be impled. Either way for faster dev I'm going to bake Juliex in, only to
//take out later.
// impl<T> Pool<T>
// where
//     T: SpawnExt,
// {
impl Pool {
    //Should accept some kind of config.
    //TODO we max_inboundht not always need a listener in this pool, so it doesn't have to be
    //constructed as a generic since it would just be useless initalization. Actually, scratch
    //that, we can just use the builder pattern here and we should be completely fine.
    pub fn new(config: PoolConfig) -> Result<Pool> {
        //Probably grab this from hosts.json file.
        let peer_list = PeerList::new();
        let peer_store = PeerStore::new();

        Ok(Pool {
            config,
            peers: Arc::new(Mutex::new(peer_list)),
            connected: false,
            store: Arc::new(peer_store),
        })
    }

    //Main
    pub async fn start(&self) -> Result<()> {
        // 1. Make sure all options are initalized (will probably come in the new function.
        // 2. Attempt to read the Peers.dat/hosts.json file. If it works, load up the peer store.
        // 3. Start the listening function (if listening) (async)
        // 4. Start the new connections function (async)
        // 5. Start DNS Seed function (if dns seeds are enabled (async)
        // TODO spawn a task that just does feeler connections
        // Function for dumping network addresses to peer store to be saved to the file. This
        // should either be a function peer_store, or here.
        let pool_pointer = self.clone();

        juliex::spawn(async move {pool_pointer.open_connections().await.unwrap()});

        //Infinite loop for start.
        loop {

        }

        Ok(())
    }

    async fn open_connections(&self) -> Result<()> {
        //infinite loop
        loop {
            //Sleep here for some amount of time.
            let need = self.config.max_outbound - self.peers.lock().await.outbound;

            //If we don't need any then reset loop
            if need == 0 {
                continue;
            }

            info!(
                "Refilling peers ({}/{}).",
                self.peers.lock().await.outbound, self.config.max_outbound
            );

            //Sleep for 500 milliseconds
            Delay::new(Duration::from_millis(500)).await?;

            info!("Slept for 500ms!");

            //Resolve any collisions in the peer store.
            self.store.resolve_collisions().await?;

            let start = Time::now();
            let mut attempts: u32 = 0;
            let feeler = false;
            let mut address_connect: Option<NetAddress> = None;

            loop {
                let mut data_locked = self.store.select_tried_collision().await;

                if !feeler || data_locked.is_none() {
                    data_locked = self.store.select(feeler).await;
                }

                //Get the lock on the data
                //TODO remove the unwrap here and put it into a break if statment. Probably if let
                //statement here.
                //TODO so ugly, fix this.
                let data_unwrapped = data_locked.unwrap();
                let data = data_unwrapped.lock().await;

                //TODO we aren't checking groups right now as how we are returning them is bad.
                // let connected_groups = self.connected_groups.lock().await;

                // if !feeler || self.connected_groups

                if !data.address.is_valid() || data.address.is_local() {
                    break;
                }

                attempts += 1;

                if attempts > 100 {
                    break;
                }

                //TODO needs to impl on pool, not on net address
                // if !data.address.is_reachable() {
                //     continue;
                // }

                if start - data.last_try < 600 && attempts < 30 {
                    continue;
                }

                //TODO these go in protocol
                // if !feeler && !self.has_all_desirable_service_flags(address.services) {
                //     continue;
                // } else if feeler && may_have_useful_address_db(address.services) {
                //     continue;
                // }

                //TODO need default port -> Probably grab from network.
                // if data.address.port != default_port && attempts < 50 {
                //     continue;
                // }

                address_connect = Some(data.address);
                break;
            };

            if address_connect.is_none() {
                //TODO another loop?
                break;
            }

            if address_connect.unwrap().is_valid() {
                //if feeler {
                //    //TODO add a random amount of noise before connection to avoid synchronization.
                //    }
                //Make the connection to the address probably means new peer.
                //TODO need to impl key
                //TODO need to impl network (or remove it from this function)
                //Might not want to throw the error here, and just continue.
                juliex::spawn(async move {
                let peer = Peer::connect(address_connect.unwrap(), [0; 32], Network::Testnet).await.unwrap();
                    // peer.handle()

                });
            }

            //Find a valid address.
            //Open the connection.
            //If successful, then let loop occur.
        }
        Ok(())

        // this.logger.debug('Refilling peers (%d/%d).',
        //   this.peers.outbound,
        //   this.options.maxOutbound);

        // for (let i = 0; i < need; i++)
        //   this.addOutbound();

        //pub async fn listen(&self) -> Result<()> {
        //    //Open up the server, and attempt to bind to the port.
        //    //If that is successful, then for each new connection check if we are below the acceptable
        //    //new connections
        //    //If we are, then create a new peer from that connection and spawn them into a new task.
        //    //For the above, I think we should do the new handshake in that function, and then spawn
        //    //the peer into it's own handler.
        //    let mut listener = TcpListener::bind(&"127.0.0.1:7878".parse().unwrap())?;
        //    let mut incoming = listener.incoming();

        //    info!("Listening on 127.0.0.1:7878");

        //    while let Some(stream) = incoming.next().await {
        //        let stream = stream?;
        //        let addr = stream.peer_addr()?;
        //        let peer =
        //        self.executor.spawn(async move {
        //                info!("Accepting stream from: {}", addr);

        //                recite_shakespeare(stream).await.unwrap();

        //                println!("Closing stream from: {}", addr);
        //            }).unwrap()
        //    }
        //}

        //Connects the pool to nodes/etc.
        //pub async fn connect(&mut self) -> Result<()> {
        //    if self.connected {
        //        return;
        //    }

        //    //Open up the hosts file.
        //    //This should be handled in PeerList::new up above, so I think we can skip this.

        //    // self.discover_gateway().await?;
        //    // TODO remove this and replace with PR from bitcoin core.
        //    // self.discover_external().await?;
        //    self.discover_seeds(false).await?;

        //    // this.fillOutbound();

        //    // await this.listen();

        //    // this.startTimer();

        //    // this.connected = true;
        //    //
        //}

        //TODO can't do until we finish host file.
        // pub async fn discover_seeds(&mut self) {}

        // if (this.hosts.dnsSeeds.length === 0)
        //   return;

        // const max = Math.min(2, this.options.maxOutbound);
        // const size = this.hosts.size();

        // let total = 0;
        // for (let peer = this.peers.head(); peer; peer = peer.next) {
        //   if (!peer.outbound)
        //     continue;

        //   if (peer.connected) {
        //     if (++total > max)
        //       break;
        //   }
        // }

        // if (size === 0 || (checkPeers && total < max)) {
        //   this.logger.warning('Could not find enough peers.');
        //   this.logger.warning('Hitting DNS seeds...');

        //   await this.hosts.discoverSeeds();

        //   this.logger.info(
        //     'Resolved %d hosts from DNS seeds.',
        //     this.hosts.size() - size);

        //   this.refill();
        // }

        //Likely going to remove this for the bitcoin core path.
        // pub async fn discover_external(&mut self) -> Result<()> {

        // }

        //TODO UPNP options. Not going to implement for now
        //See: https://docs.rs/igd/0.9.0/igd/
        // pub async fn discover_gateway() {
        //     unimplemented!();
        // }

        //Main function of the pool -> Initalizes everything for the pool, and then triggers 3 threads
        //1. Inital sync thread.
        //2. Listen thread (If listen is configured).
        //3. Get Outbound Peers thread.
        //pub fn run(&mut self) {
        //    //Reset the header chain here.

        //    //I think we should have a channel that this pool loops on and listens to. Any messages
        //    //That require the pool to act on are read through and then handled accordingly.
        //    //self.thread_pool.run( async {
        //    //    loop {
        //    //        //Check if stop state from above has been called. TODO
        //    //    }
        //    //});
        //}

        //ALso run in it's own thread
        //This is only run if the node wants to listen for inbound peers.
        // pub fn listen(&mut self) {
        //     unimplemented!();
        // }

        //Unclear if this function is needed.
        //TODO I think we can remove this function since we don't have checkpoints in Handshake.
        //pub fn reset_chain(&mut self) {
        //    //Current header tip
        //    self.header_tip = None;
        //    //list of header entries. TODO
        //    self.header_chain = vec![];
        //    //I'm guessing this is the next header to be added
        //    self.header_next = None;
        //}

        //Run it in it's own thread -> Should run indefinitely.
        // pub fn sync(&mut self) {}

        //pub async fn fill_outbound(&mut self) {
        //    let need = self.config.max_outbound - self.peers.outbound;

        //    if self.peers.loader.is_none() {
        //        self.add_loader();
        //    }

        //    //Double check this logic here
        //    if need == 0 {
        //        return;
        //    }

        //    for _ in 0..need {
        //        // self.add_outbound();
        //    }
        //}

        //pub fn add_loader(&mut self) {
        //    if self.peers.loader.is_some() {
        //        return;
        //    }

        //    //Iterate over peer list.
        //    let mut iter = self.peers.get_connected().into_iter();

        //    //Iterate over existing peers to find any that could be set as loaders, if not continue.
        //    while let Some(peer) = iter.next() {
        //        if !peer.is_outbound() {
        //            continue;
        //        }

        //        self.set_loader(peer);

        //        return;
        //    }

        //    //Grab a random host from the peerlist.
        //    let address = match self.peers.get_host() {
        //        Ok(addr) => addr,
        //        Err(_) => return,
        //    };

        //    // let peer = this.createOutbound(addr);

        //    // this.logger.info('Adding loader peer (%s).', peer.hostname());

        //    // this.peers.add(peer);

        //    // this.setLoader(peer);
        //    // }
        //}

        //pub fn set_loader(&mut self, mut peer: Arc<Peer>) -> Result<()> {
        //    //TODO I don't think we need these checks, but leaving them here for now.
        //    // assert(peer.outbound);
        //    // assert(!this.peers.load);
        //    // assert(!peer.loader);

        //    //These both need to be RwLock TODO
        //    peer.set_loader(true)?;
        //    self.peers.set_loader_peer(peer);

        //    //TODO start the syncing.
        //    //self.send_sync(peer);

        //    // this.emit('loader', peer);
        //    Ok(())
        //}

        // pub fn create_outbound(&mut self, addr: NetAddress) -> Result<()> {

        // }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::executor::block_on;

    #[test]
    fn test_pool_run() {
        let config = PoolConfig {
            max_outbound: 8,
            max_inbound: 8,
        };

        let pool = Pool::new(config).unwrap();
        //TODO after a timeout, trigger the stop state.

        block_on(async { pool.start().await});
    }
}
