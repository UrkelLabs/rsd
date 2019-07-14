use crate::NetAddress;
use crate::Result;
use extended_primitives::Uint256;
use handshake_protocol::network::Network;
use handshake_types::Time;
use log::info;
use rand::{thread_rng, Rng};
use std::collections::HashMap;
use std::sync::RwLock;

mod common;
mod peer_data;

use peer_data::PeerData;

use common::{
    BUCKET_SIZE, BUCKET_SIZE_LOG2, NEW_BUCKET_COUNT, REPLACEMENT_HOURS, TEST_WINDOW,
    TRIED_BUCKET_COUNT, TRIED_BUCKET_COUNT_LOG2,
};

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
    new: [[Option<u32>; NEW_BUCKET_COUNT]; BUCKET_SIZE],
    //Convert to internal time.
    last_good: Time,
    tried_collisions: Vec<u32>,
    key: Uint256,
    network: Network,
    //WHAT IS THIS? TODO
    // address: NetAddress,
    // dns_seeds: Vec<SocketAddr>
    // dns_nodes: Vec<SocketAddr>
    // peer_data: RwLock<HashMap<NetAddress, PeerData>>,
    // fresh: RwLock<Vec<HashMap<NetAddress, PeerData>>>,
    //TODO probably don't need. can just say fresh.len()
    // total_fresh: u32,
    // used: RwLock<Vec<Vec<NetAddress>>>,
    // total_used: u32,
    // nodes: RwLock<Vec<NetAddress>>,
    // needs_flush: bool,
    //TODO add banned list.
    //     this.timer = null;
    //     this.needsFlush = false;
    //     this.flushing = false;
    //     this.added = false;
}

//I think the goal is to keep this file as a json file to allow easy parsing.
impl PeerStore {
    //TODO 1. Probably don't even need network. 2. This should be a config option, OR
    //Should be builder pattern.
    pub fn new(network: Network) -> PeerStore {
        let id_count = 0;
        let map_info = HashMap::new();
        let map_address = HashMap::new();
        let random = Vec::new();
        let tried_count = 0;
        let tried = [[None; TRIED_BUCKET_COUNT]; BUCKET_SIZE];
        let new_count = 0;
        let new = [[None; NEW_BUCKET_COUNT]; BUCKET_SIZE];
        //Initially at 1 so that "never" is strictly worse.
        let last_good = Time::new() + 1;
        let tried_collisions = Vec::new();
        //TODO
        // let key = Uint256::random();
        let key = Uint256::default();

        PeerStore {
            id_count,
            map_info,
            map_address,
            random,
            tried_count,
            tried,
            new,
            new_count,
            last_good,
            tried_collisions,
            key,
            network,
        }
    }

    //Returns an address that is attempting to be evicted by another address.
    pub fn select_tried_collision(&self) -> Result<Option<NetAddress>> {
        Ok(None)
    }

    //Probably doesn't need to return a result.
    pub fn resolve_collisions(&mut self) -> Result<()> {
        let collisions = self.tried_collisions.iter();

        for (i, collision) in collisions.enumerate() {
            let id_new = collision;
            let mut erase_collision = false;

            //Can always switch this to match map_info.get()
            //None => erase collision = true
            //Some => return the var.
            //TODO change to map_data since we use peer_data.
            //TODO change this to if let -> has cleaner syntax and less indenting.
            match self.map_info.get(id_new) {
                None => erase_collision = true,
                Some(data_new) => {
                    //Find the tried bucket to move the entry to.
                    let tried_bucket = data_new.get_tried_bucket(self.key);
                    let tried_bucket_position =
                        data_new.get_bucket_position(self.key, false, tried_bucket);

                    //If the peer data no longer has a valid address, remove the collision.
                    if !data_new.is_valid() {
                        erase_collision = true;
                    } else {
                        //TODO change this to if let -> has cleaner syntax and less indenting.
                        match self.tried[tried_bucket][tried_bucket_position] {
                            None => {
                                // self.good(data_new, false, Time::now());
                                data_new.mark_good();
                                self.make_tried(data_new, id_new);
                                erase_collision = true;
                            }
                            Some(id_old) => {
                                //TODO make map_info a vector, and then rename to map_data
                                let data_old = self.map_info.get(&id_old).unwrap();
                                //If the old address has connected within the last X hours, we keep it.
                                if Time::now() - data_old.last_success < REPLACEMENT_HOURS * 60 * 60
                                {
                                    erase_collision = true;
                                //Attempted to connect and failed in last X hours
                                } else if Time::now() - data_old.last_try
                                    < REPLACEMENT_HOURS * 60 * 60
                                {
                                    if Time::now() - data_old.last_try > 60 {
                                        // info!("Replacing {} with {} in tried table", info_old.address, info_new.address);
                                        data_new.mark_good();
                                        self.make_tried(data_new, id_new);
                                        erase_collision = true;
                                    }
                                } else if Time::now() - data_new.last_success > TEST_WINDOW {
                                    // If collision isn't resolved in some reasonable amount of time, then we
                                    // just evict the old entry - connections are not working to it for some
                                    // reason.
                                    // TODO implement display on peer_data, just wrap net_addresses display.
                                    // info!("Unable to test; replacing {} with {} in tried table anyway", info_old.address, info_new.address);
                                    data_new.mark_good();
                                    self.make_tried(data_new, id_new);
                                    erase_collision = true;
                                }
                            }
                        }
                    }
                }
            }

            //Delete this collison, and then continue the iterator.
            if erase_collision {
                self.tried_collisions.remove(i);
            }
        }
        Ok(())
    }

    pub fn make_tried(&self, data: &PeerData, id: &u32) {
        for i in 0..NEW_BUCKET_COUNT {
            let position = data.get_bucket_position(self.key, true, i);
            if let Some(old_id) = self.new[i][position] {
                if old_id == *id {
                    self.new[i][position] == None;
                    data.ref_count -= 1;
                }
            }
        }
        self.new_count -= 1;

        assert!(data.ref_count == 0);

        let bucket = data.get_tried_bucket(self.key);
        let position = data.get_bucket_position(self.key, false, bucket);

        match self.tried[bucket][position] {
            None => {}
            Some(id_evict) => {
                assert!(self.map_info.contains_key(&id_evict));
                //TODO remove unwraps
                let data_old = self.map_info.get_mut(&id_evict).unwrap();

                //Remove the to-be-evicted item from the tried set.
                data_old.in_tried = false;
                self.tried[bucket][position] == None;
                self.tried_count -= 1;

                //Find which new bucket this address belongs to.
                let new_bucket = data_old.get_new_bucket(self.key);
                let new_position = data_old.get_bucket_position(self.key, true, new_bucket);
                self.clear_new(new_bucket, new_position);
                assert!(self.new[new_bucket][new_position].is_none());

                data_old.ref_count = 1;
                self.new[new_bucket][new_position] = Some(id_evict);
                self.new_count += 1;
            }
        }

        assert!(self.tried[bucket][position] == None);

        self.tried[bucket][position] = Some(id.clone());
        self.tried_count += 1;
        data.in_tried = true;
    }

    // if there is an entry in the specified bucket, delete it.
    pub fn clear_new(&self, bucket: usize, position: usize) {
        //TODO rewrite this as these may already be empty.
        let id_delete = self.new[bucket][position].unwrap();
        let data_delete = self.map_info.get(&id_delete).unwrap();
        assert!(data_delete.ref_count > 0);
        data_delete.ref_count -= 1;
        self.new[bucket][position] = None;
        if data_delete.ref_count == 0 {
            self.delete(&id_delete);
        }
    }

    pub fn delete(&self, id: &u32) {
        //TODO finish unwraps
        assert!(self.map_info.contains_key(id));
        let data = self.map_info.get(id).unwrap();
        //Panic if trying to delete an address inside of tried.
        assert!(!data.in_tried);
        //Panic if data is still referenced in the maps.
        assert!(data.ref_count == 0);

        self.swap_random(data.random_position, self.random.len() - 1);
        //TODO double check that this works.
        self.random.truncate(self.random.len() - 1);
        self.map_address.remove_entry(&data.address);
        self.map_info.remove_entry(id);
    }

    pub fn swap_random(&self, position: usize, position_2: usize) {
        if position == position_2 {
            return;
        }

        assert!(position < self.random.len() && position_2 < self.random.len());

        let id = self.random[position];
        let id2 = self.random[position_2];

        //TODO can be re-written with Matches I believe.
        assert!(self.map_info.contains_key(&id));
        assert!(self.map_info.contains_key(&id2));

        self.map_info.get_mut(&id).unwrap().random_position = position_2;
        self.map_info.get_mut(&id2).unwrap().random_position = position;

        self.random[position] = id2;
        self.random[position_2] = id;
    }

    // pub fn good(&self, data: PeerData, test_before_evict: bool, time: Time) {
    //     let id: u32;
    //     let last_good = time;
    // }
    // void CAddrMan::Good_(const CService& addr, bool test_before_evict, int64_t nTime)
    // {
    //     int nId;

    //     nLastGood = nTime;

    //     CAddrInfo* pinfo = Find(addr, &nId);

    //     // if not found, bail out
    //     if (!pinfo)
    //         return;

    //     CAddrInfo& info = *pinfo;

    //     // check whether we are talking about the exact same CService (including same port)
    //     if (info != addr)
    //         return;

    //     // update info
    //     info.nLastSuccess = nTime;
    //     info.nLastTry = nTime;
    //     info.nAttempts = 0;
    //     // nTime is not updated here, to avoid leaking information about
    //     // currently-connected peers.

    //     // if it is already in the tried set, don't do anything else
    //     if (info.fInTried)
    //         return;

    //     // find a bucket it is in now
    //     int nRnd = insecure_rand.randrange(ADDRMAN_NEW_BUCKET_COUNT);
    //     int nUBucket = -1;
    //     for (unsigned int n = 0; n < ADDRMAN_NEW_BUCKET_COUNT; n++) {
    //         int nB = (n + nRnd) % ADDRMAN_NEW_BUCKET_COUNT;
    //         int nBpos = info.GetBucketPosition(nKey, true, nB);
    //         if (vvNew[nB][nBpos] == nId) {
    //             nUBucket = nB;
    //             break;
    //         }
    //     }

    //     // if no bucket is found, something bad happened;
    //     // TODO: maybe re-add the node, but for now, just bail out
    //     if (nUBucket == -1)
    //         return;

    //     // which tried bucket to move the entry to
    //     int tried_bucket = info.GetTriedBucket(nKey);
    //     int tried_bucket_pos = info.GetBucketPosition(nKey, false, tried_bucket);

    //     // Will moving this address into tried evict another entry?
    //     if (test_before_evict && (vvTried[tried_bucket][tried_bucket_pos] != -1)) {
    //         // Output the entry we'd be colliding with, for debugging purposes
    //         auto colliding_entry = mapInfo.find(vvTried[tried_bucket][tried_bucket_pos]);
    //         LogPrint(BCLog::ADDRMAN, "Collision inserting element into tried table (%s), moving %s to m_tried_collisions=%d\n", colliding_entry != mapInfo.end() ? colliding_entry->second.ToString() : "", addr.ToString(), m_tried_collisions.size());
    //         if (m_tried_collisions.size() < ADDRMAN_SET_TRIED_COLLISION_SIZE) {
    //             m_tried_collisions.insert(nId);
    //         }
    //     } else {
    //         LogPrint(BCLog::ADDRMAN, "Moving %s to tried\n", addr.ToString());

    //         // move nId to the tried tables
    //         MakeTried(info, nId);
    //     }
    // }

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

//Needs to print a random boolean
fn random(max: u32) -> u32 {
    //TODO double check inclusiveness
    thread_rng().gen_range(0, max)
}
