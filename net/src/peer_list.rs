use std::collections::HashMap;
use std::sync::{Arc, RwLock};
//TODO move these to first class imports.
use crate::error::Error;
use crate::net_address::NetAddress;
use crate::peer::Peer;
use crate::Result;

//TODO this might go inside of the peer file.
//TODO implement PeerStore (Hostlist) and include in most functions.
pub struct PeerList {
    list: RwLock<HashMap<NetAddress, Arc<Peer>>>,
    pub outbound: u32,
    pub loader: Option<Arc<Peer>>,
    //TODO keep local and banned in peer list since they are not persisted in a db.
    //     this.local = new Map();
    //     this.banned = new Map();
}

impl PeerList {
    pub fn new() -> PeerList {
        PeerList {
            list: RwLock::new(HashMap::new()),
            outbound: 0,
            loader: None,
        }
    }

    //TODO implement storing this in PeerStore (Hostlist)
    pub fn add_connected(&self, peer: Arc<Peer>) -> Result<()> {
        //TODO 2 notes.
        //1. This is going to block if we can't acquire access to it -> I think that it is unlikely
        //   that this will block for long, but I see many other projects utilizing try_write_for
        //   on a timeout.
        //   2. In order to get try_write_for we would need to use Parking_lot's re-implementation
        //      of RwLock. It's also apparently much faster. Look into if that is something worth
        //      pursuing -> For now, we will just use write, and block on non-write access.
        let mut peers = match self.list.write() {
            Ok(peers) => peers,
            //Can't wrap trylock error so we have to do this.
            Err(_) => return Err(Error::LockError),
        };

        //TODO save this peer into the hostlist/peerstore as well.
        peers.insert(peer.info.address, peer.clone());

        Ok(())
    }

    pub fn get_connected(&self) -> Vec<Arc<Peer>> {
        let mut peers = match self.list.write() {
            Ok(peers) => peers,
            Err(_) => return vec![],
        };

        let mut res: Vec<Arc<Peer>> = peers
            .values()
            .filter(|p| p.is_connected())
            .cloned()
            .collect();

        //TODO optional shuffling of connected peers.
        // 		res.shuffle(&mut thread_rng());
        // 		res
        res
    }

    pub fn set_loader_peer(&mut self, peer: Arc<Peer>) {
        self.loader = Some(peer);
    }

    //Return a random peer from the peer store NOT the connected peers.
    //Verify all information of a peer when pulling from the file. If key is not valid (or any
    //other value), remove it from the list and move to the next peer. If none are left return
    //none or err.
    pub fn get_host(&self) -> Result<()> {
        //TODO
        unimplemented!();
        // TODO verify key when pulling
        // if (!secp256k1.publicKeyVerify(addr.key)) {
        //   this.logger.info('Removing addr - invalid pubkey (%s).', addr.hostname);
        //   this.hosts.remove(addr.hostname);
        //   return;
        // }

        Ok(())
    }
}
