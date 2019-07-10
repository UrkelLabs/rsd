use crate::common::MAX_REFS;
use crate::NetAddress;
use crate::Result;
use chrono::{DateTime, TimeZone, Utc};
use handshake_protocol::network::Network;
use log::{info, warn};
use rand::thread_rng;
use std::collections::HashMap;
use std::sync::RwLock;

mod common;
mod peer_data;

// use common::{BUCKET_SIZE, NEW_BUCKET_COUNT, TRIED_BUCKET_COUNT};
use peer_data::PeerData;
// use crate::peer_store::peer_data

use crate::peer_store::common::{BUCKET_SIZE, NEW_BUCKET_COUNT, TRIED_BUCKET_COUNT};

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
    tried: [[NetAddress; TRIED_BUCKET_COUNT]; BUCKET_SIZE],
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
}

// fn random(max: u32) -> u32 {
//     thread_rng().gen_range(0, max)
// }
