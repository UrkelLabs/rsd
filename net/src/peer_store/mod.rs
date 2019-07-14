use crate::common::MAX_REFS;
use crate::NetAddress;
use crate::Result;
use chrono::{DateTime, TimeZone, Utc};
use handshake_protocol::network::Network;
use log::{info, warn};
use rand::{thread_rng, Rng};
use std::collections::HashMap;
use std::sync::RwLock;

mod common;
mod peer_data;

// use common::{BUCKET_SIZE, NEW_BUCKET_COUNT, TRIED_BUCKET_COUNT};
use peer_data::PeerData;
// use crate::peer_store::peer_data

use crate::peer_store::common::{
    BUCKET_SIZE, BUCKET_SIZE_LOG2, NEW_BUCKET_COUNT, TRIED_BUCKET_COUNT, TRIED_BUCKET_COUNT_LOG2,
};

// pub enum Scores {
//     None = 0,
//     If = 1,
//     Bind = 2,
//     Upnp = 3,
//     Dns = 3,
//     Manual = 4,
//     Max = 5,
// }

//TODO PeerStore should just accept a config of a trait that implements Datastore
//That way anyone or any plugin can just provide their own store and it will get passed down to
//this. SO if someone wants to store peers in leveldb. They can do that.
//TODO put name in here as an option.
//Change Peer Store config to Peer Store Builder....
//Then we can set these as defaults.
pub struct PeerStoreConfig {
    max_entries: u32,
    max_buckets: u32,
}

//TODO impl default on peer store config.

// this.banTime = common.BAN_TIME;
//  this.maxBuckets = 20;
// this.maxEntries = 50;
//
// TODO need to impl our own time variable in Handshake_Types. Look at what Grin does.
//  //! last used nId

////! table with information about all nIds
//std::map<int, CAddrInfo> mapInfo GUARDED_BY(cs);

////! find an nId based on its network address
//std::map<CNetAddr, int> mapAddr GUARDED_BY(cs);

////! randomly-ordered vector of all nIds
//std::vector<int> vRandom GUARDED_BY(cs);

//// number of "tried" entries
//int nTried GUARDED_BY(cs);

////! list of "tried" buckets
//int vvTried[ADDRMAN_TRIED_BUCKET_COUNT][ADDRMAN_BUCKET_SIZE] GUARDED_BY(cs);

////! number of (unique) "new" entries
//int nNew GUARDED_BY(cs);

////! list of "new" buckets
//int vvNew[ADDRMAN_NEW_BUCKET_COUNT][ADDRMAN_BUCKET_SIZE] GUARDED_BY(cs);

////! last time Good was called (memory only)
//int64_t nLastGood GUARDED_BY(cs);

////! Holds addrs inserted into tried table that collide with existing entries. Test-before-evict discipline used to resolve these collisions.
//std::set<int> m_tried_collisions;

pub struct PeerStore {
    //Last used ID
    //TODO possibly make this it's own type.
    //mutex?
    id_count: u32,
    //mutex
    map_info: HashMap<u32, PeerData>,
    //mutex
    map_address: HashMap<NetAddress, u32>,
    //Random vector of all ids -> mutex
    random: Vec<u32>,
    //Amount of tried entries.
    tried_count: u32,
    //double check symantics on this.
    //mutex
    tried: [[Option<u32>; TRIED_BUCKET_COUNT]; BUCKET_SIZE],
    new_count: u32,
    //mutex
    new: [[NetAddress; NEW_BUCKET_COUNT]; BUCKET_SIZE],
    //Convert to internal time.
    last_good: DateTime<Utc>,
    tried_collisions: Vec<u32>,
    network: Network,
    //WHAT IS THIS? TODO
    address: NetAddress,
    // dns_seeds: Vec<SocketAddr>
    // dns_nodes: Vec<SocketAddr>
    peer_data: RwLock<HashMap<NetAddress, PeerData>>,
    fresh: RwLock<Vec<HashMap<NetAddress, PeerData>>>,
    //TODO probably don't need. can just say fresh.len()
    total_fresh: u32,
    used: RwLock<Vec<Vec<NetAddress>>>,
    total_used: u32,
    nodes: RwLock<Vec<NetAddress>>,
    needs_flush: bool,
    //TODO add banned list.
    //     this.timer = null;
    //     this.needsFlush = false;
    //     this.flushing = false;
    //     this.added = false;
}

//I think the goal is to keep this file as a json file to allow easy parsing.
impl PeerStore {
    pub fn new() {}

    //Returns an address that is attempting to be evicted by another address.
    //We do this for test-before-evict mentality. TODO it's an interesting possibility to just wrap
    //this function into the general "get_address" function, to reduce confusion.
    //If we do that, then make this not public.
    pub fn select_tried_collision(&self) -> Result<Option<NetAddress>> {
        Ok(None)
    }

    //This returns a reference directly inside of the hash map. Not sure if that's the
    //functionality we want from this function, so come back to this.
    pub fn select(&self, new_only: bool) -> Result<Option<&PeerData>> {
        //If the store is empty, return none.
        if self.size() == 0 {
            return Ok(None);
        };

        //If we only want new addresses, and the new addresses are empty, return none.
        if new_only && self.new_count == 0 {
            return Ok(None);
        }

        if !new_only && self.tried_count > 0 && (self.new_count == 0 || !random(1) == 0) {
            let mut chance = 1.0;
            loop {
                //Make a random usize function. so that we don't have to coax all these values.
                //TODO don't coax, use usize everywhere.
                let mut bucket = random(TRIED_BUCKET_COUNT as u32);
                let mut bucket_position = random(BUCKET_SIZE as u32);

                while self.tried[bucket as usize][bucket_position as usize].is_none() {
                    bucket = (bucket + random(TRIED_BUCKET_COUNT_LOG2 as u32))
                        % TRIED_BUCKET_COUNT as u32;
                    bucket_position =
                        (bucket_position + random(BUCKET_SIZE_LOG2)) % BUCKET_SIZE as u32;
                }

                let id = self.tried[bucket as usize][bucket_position as usize];

                let info = self.map_info.get(&id.unwrap());

                //TODO cleaner way of doing this, probably with map error.
                if info.is_none() {
                    //throw error
                }

                //TODO clean up all the coaxings.

                if (random(30) as f64) < chance * info.unwrap().get_chance() * ((1 << 30) as f64) {
                    return Ok(info);
                }

                chance *= 1.2;
            }
        }

        Ok(None)
    }

    // if (!newOnly &&
    //    (nTried > 0 && (nNew == 0 || insecure_rand.randbool() == 0))) {
    //     // use a tried node
    //     double fChanceFactor = 1.0;
    //     while (1) {
    //         int nId = vvTried[nKBucket][nKBucketPos];
    //         assert(mapInfo.count(nId) == 1);
    //         CAddrInfo& info = mapInfo[nId];
    //         if (insecure_rand.randbits(30) < fChanceFactor * info.GetChance() * (1 << 30))
    //             return info;
    //         fChanceFactor *= 1.2;
    //     }
    // } else {
    //     // use a new node
    //     double fChanceFactor = 1.0;
    //     while (1) {
    //         int nUBucket = insecure_rand.randrange(ADDRMAN_NEW_BUCKET_COUNT);
    //         int nUBucketPos = insecure_rand.randrange(ADDRMAN_BUCKET_SIZE);
    //         while (vvNew[nUBucket][nUBucketPos] == -1) {
    //             nUBucket = (nUBucket + insecure_rand.randbits(ADDRMAN_NEW_BUCKET_COUNT_LOG2)) % ADDRMAN_NEW_BUCKET_COUNT;
    //             nUBucketPos = (nUBucketPos + insecure_rand.randbits(ADDRMAN_BUCKET_SIZE_LOG2)) % ADDRMAN_BUCKET_SIZE;
    //         }
    //         int nId = vvNew[nUBucket][nUBucketPos];
    //         assert(mapInfo.count(nId) == 1);
    //         CAddrInfo& info = mapInfo[nId];
    //         if (insecure_rand.randbits(30) < fChanceFactor * info.GetChance() * (1 << 30))
    //             return info;
    //         fChanceFactor *= 1.2;
    //     }
    // }

    // Return the number of unique address across all buckets.
    pub fn size(&self) -> u32 {
        0
    }

    // fn set_seeds(&mut self, seeds: Vec<String>) -> Result<()> {
    //     for x in 0..seeds {
    //         self.add_seed(seeds[x]);
    //     }
    // }

    // fn add_seed(&mut self, seed_hostname: String) ->
}
// addSeed(host) {
//   const ip = IP.fromHostname(host, this.network.port);

//   if (ip.type === IP.types.DNS) {
//     // Defer for resolution.
//     this.dnsSeeds.push(ip);
//     return null;
//   }

//   if (!ip.key)
//     throw new Error('Must have identity key.');

//   const addr = NetAddress.fromHost(ip.host, ip.port, ip.key, this.network);

//   this.add(addr);

//   return addr;
// }
// setSeeds(seeds) {
//   this.dnsSeeds.length = 0;

//   for (const host of seeds)
//     this.addSeed(host);
// }
// initAdd() {
//   const options = this.options;
//   const scores = HostList.scores;

//   this.setSeeds(options.seeds);
//   this.setNodes(options.nodes);

//   this.pushLocal(this.address, scores.MANUAL);
//   this.addLocal(options.host, options.port, scores.BIND);

//   const hosts = IP.getPublic();
//   const port = this.address.port;

//   for (const host of hosts)
//     this.addLocal(host, port, scores.IF);

//   this.added = true;
// }

//pub fn new(
//    network: Network,
//    address: NetAddress,
//    db_root: &str,
//    config: PeerStoreConfig,
//) -> Result<PeerStore> {
//    let fresh = Vec::new();

//    //Create initial buckets
//    for _ in 0..config.max_buckets {
//        fresh.push(HashMap::new());
//    }

//    //Wrap fresh in lock
//    let fresh = RwLock::new(fresh);

//    let mut used = Vec::new();

//    for _ in 0..config.max_buckets {
//        used.push(Vec::new());
//    }

//    //Wrap used in lock.
//    let used = RwLock::new(used);

//    let peer_data = RwLock::new(HashMap::new());

//    let nodes = RwLock::new(Vec::new());

//    Ok(PeerStore {
//        network,
//        address,
//        fresh,
//        used,
//        total_fresh: 0,
//        total_used: 0,
//        needs_flush: false,
//        peer_data,
//        nodes,
//    })

//    //If the file exists then read it into memory.
//    //If it doesn't not exist create it.
//    //We want to load the entire thing into memory, so We'll call a helper function: parse.
//    // let hosts = OpenOptions::new()
//    //     .read(true)
//    //     .write(true)
//    //     .create(true)
//    //     .open("");
//}

//pub fn init_add() {

//    // const options = this.options;
//    // const scores = HostList.scores;

//    // this.setSeeds(options.seeds);
//    // this.setNodes(options.nodes);

//    // this.pushLocal(this.address, scores.MANUAL);
//    // this.addLocal(options.host, options.port, scores.BIND);

//    // const hosts = IP.getPublic();
//    // const port = this.address.port;

//    // for (const host of hosts)
//    // this.addLocal(host, port, scores.IF);

//    // this.added = true;<Paste>

//}

//pub fn add(&mut self, addr: NetAddress, src: Option<NetAddress>) -> Result<bool> {
//    let entry = self.peer_data.read()?.get_mut(addr);

//    //is some
//    if let Some(entry) = entry {
//        let mut penalty = 2 * 60 * 60;
//        let mut interval = 24 * 60 * 60;

//        if src.is_none() {
//            penalty = 0;
//        }

//        //What is this? TODO
//        entry.address.services |= addr.services;
//        //TODO I don't think we need ot do this
//        entry.address.services >>= 0;

//        let now = Utc.timestamp();

//        if now - addr.time < interval {
//            interval = 60 * 60;
//        }

//        if entry.address.time < addr.time - interval - penalty {
//            entry.address.time = addr.time;
//            self.needs_flush = true;
//        }

//        if entry.address.time && addr.time <= entry.address.time {
//            return Ok(false);
//        }

//        if entry.used {
//            return Ok(false);
//        }

//        //Make source ref count is not 0? TODO
//        // assert(entry.refCount > 0);

//        if entry.ref_count == MAX_REFS {
//            return Ok(false);
//        }

//        //Maybe check this? TODO theoretically should never go beyond but maybe have a check
//        //and throw an error.
//        //   assert(entry.refCount < HostList.MAX_REFS);

//        let mut factor = 1;
//        for _ in 0..entry.ref_count {
//            factor *= 2;

//            if random(factor) != 0 {
//                return Ok(false);
//            }
//        }
//    //Is none.
//    } else {
//        if self.is_full() {
//            return Ok(false);
//        }

//        //TODO doublecheck this
//        if src.is_none() {
//            src = Some(self.address);
//        }

//        entry = self.total_fresh += 1;
//    }

//    let mut bucket = self.fresh_bucket(entry);

//    // const bucket = this.freshBucket(entry);

//    // if (bucket.has(entry.key()))
//    // return false;

//    // if (bucket.size >= this.options.maxEntries)
//    // this.evictFresh(bucket);

//    // bucket.set(entry.key(), entry);
//    // entry.refCount += 1;

//    // this.map.set(entry.key(), entry);
//    // this.needsFlush = true;

//    // return true;<Paste>

//    Ok(())
//}

//pub fn is_full(&self) -> bool {
//    let max = self.config.max_buckets * self.config.max_entries;

//    self.size() >= max
//}

//pub fn size(&self) -> u32 {
//    self.total_used + self.total_fresh
//}

//// if (entry) {

//// } else {
////   if (this.isFull())
////     return false;

////   if (!src)
////     src = this.address;

////   entry = new HostEntry(addr, src);

////   this.totalFresh += 1;
//// }

//// const bucket = this.freshBucket(entry);

//// if (bucket.has(entry.key()))
////   return false;

//// if (bucket.size >= this.options.maxEntries)
////   this.evictFresh(bucket);

//// bucket.set(entry.key(), entry);
//// entry.refCount += 1;

//// this.map.set(entry.key(), entry);
//// this.needsFlush = true;

//// return true;

//pub fn set_seeds() {
//    // "node.urkel.io"

//    // addSeed(host) {
//    // const ip = IP.fromHostname(host, this.network.port);
//    // let socketaddr = parse

//    // if (ip.type === IP.types.DNS) {
//    // // Defer for resolution.
//    // this.dnsSeeds.push(ip);
//    // return null;
//    // }

//    // if (!ip.key)
//    // throw new Error('Must have identity key.');

//    // const addr = NetAddress.fromHost(ip.host, ip.port, ip.key, this.network);

//    // this.add(addr);

//    // return addr;
//    // }
//}

//Needs to print a random boolean
fn random(max: u32) -> u32 {
    //TODO double check inclusiveness
    thread_rng().gen_range(0, max)
}
