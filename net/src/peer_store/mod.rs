use crate::NetAddress;
use crate::Result;
use extended_primitives::Uint256;
use futures::lock::Mutex;
use handshake_protocol::network::Network;
use handshake_types::Time;
use log::info;
use rand::{thread_rng, Rng};
use std::boxed::Box;
use std::collections::HashMap;
use std::sync::Arc;

mod common;
mod peer_data;

use peer_data::PeerData;

use common::{
    BUCKET_SIZE, BUCKET_SIZE_LOG2, NEW_BUCKET_COUNT, REPLACEMENT_HOURS, TEST_WINDOW,
    TRIED_BUCKET_COUNT, TRIED_BUCKET_COUNT_LOG2,
};

//TODO remove all get_muts when they are returning arcs. No need.
//TODO clean up all the locks and await. I think most things should be wrapped into a getter
//function that awaits the lock for you, and that way it's a lot cleaner.
//TODO we can do the bitcoin model where the whole function is locked, so we have an unscore
//function that does the dirty work, and then a non-underscore function that secures the lock.

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

//We wrap this entire struct in Mutexs since we want different futures/threads to be able to touch
//the data at different times.
pub struct PeerStore {
    //Last used ID
    id_count: Mutex<u32>,
    map_info: Mutex<HashMap<u32, Arc<Mutex<PeerData>>>>,
    map_address: Mutex<HashMap<NetAddress, u32>>,
    random: Mutex<Vec<u32>>,
    tried_count: Mutex<u32>,
    tried: Mutex<Box<[[Option<u32>; TRIED_BUCKET_COUNT]; BUCKET_SIZE]>>,
    new_count: Mutex<u32>,
    new: Mutex<Box<[[Option<u32>; NEW_BUCKET_COUNT]; BUCKET_SIZE]>>,
    last_good: Mutex<Time>,
    tried_collisions: Mutex<Vec<u32>>,
    //Should almost never change so we don't need a mutex here.
    key: Uint256,
}

//I think the goal is to keep this file as a json file to allow easy parsing.
impl PeerStore {
    pub fn new() -> PeerStore {
        let id_count = Mutex::new(0);
        let map_info = Mutex::new(HashMap::new());
        let map_address = Mutex::new(HashMap::new());
        let random = Mutex::new(Vec::new());
        let tried_count = Mutex::new(0);
        let tried = Mutex::new(Box::new([[None; TRIED_BUCKET_COUNT]; BUCKET_SIZE]));
        let new_count = Mutex::new(0);
        let new = Mutex::new(Box::new([[None; NEW_BUCKET_COUNT]; BUCKET_SIZE]));
        //Initially at 1 so that "never" is strictly worse.
        let last_good = Mutex::new(Time::new() + 1);
        let tried_collisions = Mutex::new(Vec::new());
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
        }
    }

    //Returns an address that is attempting to be evicted by another address.
    pub async fn select_tried_collision(&self) -> Option<Arc<Mutex<PeerData>>> {
        //Grab the lock for tried collisions.
        let mut tried_collisions = self.tried_collisions.lock().await;

        if tried_collisions.len() == 0 {
            return None;
        }

        //Select a random element from the collisions
        //TODO have that random_size function
        let index = random(tried_collisions.len() as u32);

        let id_new = tried_collisions[index as usize];

        //Get map info lock
        let map_info = self.map_info.lock().await;

        match map_info.get(&id_new) {
            None => {
                tried_collisions.remove(index as usize);
                None
            }
            Some(new_data_locked) => {
                let new_data = new_data_locked.lock().await;
                let tried_bucket = new_data.get_tried_bucket(self.key);
                let position = new_data.get_bucket_position(self.key, false, tried_bucket);

                let tried = self.tried.lock().await;

                let id_old = tried[tried_bucket][position].unwrap();

                Some(map_info.get(&id_old).unwrap().clone())
            }
        }
    }

    //Probably doesn't need to return a result.
    //TODO remove result if we can.
    pub async fn resolve_collisions(&self) -> Result<()> {
        let tried_collisions = self.tried_collisions.lock().await;
        let collisions = tried_collisions.iter();
        // let collisions = self.tried_collisions.lock().await.iter();

        for (i, collision) in collisions.enumerate() {
            let id_new = collision;
            let mut erase_collision = false;

            //Can always switch this to match map_info.get()
            //None => erase collision = true
            //Some => return the var.
            //TODO change to map_data since we use peer_data.
            //TODO change this to if let -> has cleaner syntax and less indenting.
            // if let Some(data_new) = self.map_info.get_mut(id_new) {}
            match self.map_info.lock().await.get(id_new) {
                None => erase_collision = true,
                Some(data_new_locked) => {
                    let mut data_new = data_new_locked.lock().await;

                    //Find the tried bucket to move the entry to.
                    let tried_bucket = data_new.get_tried_bucket(self.key);
                    let tried_bucket_position =
                        data_new.get_bucket_position(self.key, false, tried_bucket);

                    //If the peer data no longer has a valid address, remove the collision.
                    if !data_new.is_valid() {
                        erase_collision = true;
                    } else {
                        //TODO change this to if let -> has cleaner syntax and less indenting.
                        match self.tried.lock().await[tried_bucket][tried_bucket_position] {
                            None => {
                                // self.good(data_new, false, Time::now());
                                data_new.mark_good();
                                self.make_tried(data_new_locked.clone(), id_new.clone())
                                    .await;
                                erase_collision = true;
                            }
                            Some(id_old) => {
                                //TODO make map_info a vector, and then rename to map_data
                                let map_info = self.map_info.lock().await;
                                let data_old_locked = map_info.get(&id_old).unwrap();
                                let data_old = data_old_locked.lock().await;
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
                                        self.make_tried(data_new_locked.clone(), id_new.clone())
                                            .await;
                                        erase_collision = true;
                                    }
                                } else if Time::now() - data_new.last_success > TEST_WINDOW {
                                    // If collision isn't resolved in some reasonable amount of time, then we
                                    // just evict the old entry - connections are not working to it for some
                                    // reason.
                                    // TODO implement display on peer_data, just wrap net_addresses display.
                                    // info!("Unable to test; replacing {} with {} in tried table anyway", info_old.address, info_new.address);
                                    data_new.mark_good();
                                    self.make_tried(data_new_locked.clone(), id_new.clone())
                                        .await;
                                    erase_collision = true;
                                }
                            }
                        }
                    }
                }
            }

            //Delete this collison, and then continue the iterator.
            if erase_collision {
                self.tried_collisions.lock().await.remove(i);
            }
        }
        Ok(())
    }

    pub async fn make_tried(&self, data_locked: Arc<Mutex<PeerData>>, id: u32) {
        let mut data = data_locked.lock().await;

        for i in 0..NEW_BUCKET_COUNT {
            let position = data.get_bucket_position(self.key, true, i);
            if let Some(old_id) = self.new.lock().await[i][position] {
                if old_id == id {
                    self.new.lock().await[i][position] == None;
                    data.ref_count -= 1;
                }
            }
        }
        *self.new_count.lock().await -= 1;

        assert!(data.ref_count == 0);

        let bucket = data.get_tried_bucket(self.key);
        let position = data.get_bucket_position(self.key, false, bucket);

        match self.tried.lock().await[bucket][position] {
            None => {}
            Some(id_evict) => {
                assert!(self.map_info.lock().await.contains_key(&id_evict));
                //TODO remove unwraps
                let map_info = self.map_info.lock().await;
                let data_old_locked = map_info.get(&id_evict).unwrap();
                let mut data_old = data_old_locked.lock().await;

                //Remove the to-be-evicted item from the tried set.
                data_old.in_tried = false;
                self.tried.lock().await[bucket][position] == None;
                *self.tried_count.lock().await -= 1;

                //Find which new bucket this address belongs to.
                let new_bucket = data_old.get_new_bucket(self.key);
                let new_position = data_old.get_bucket_position(self.key, true, new_bucket);
                self.clear_new(new_bucket, new_position).await;
                assert!(self.new.lock().await[new_bucket][new_position].is_none());

                data_old.ref_count = 1;
                self.new.lock().await[new_bucket][new_position] = Some(id_evict);
                *self.new_count.lock().await += 1;
            }
        }

        assert!(self.tried.lock().await[bucket][position] == None);

        self.tried.lock().await[bucket][position] = Some(id.clone());
        *self.tried_count.lock().await += 1;
        data.in_tried = true;

        return;
    }

    // if there is an entry in the specified bucket, delete it.
    pub async fn clear_new(&self, bucket: usize, position: usize) {
        //TODO rewrite this as these may already be empty.
        let id_delete = self.new.lock().await[bucket][position].unwrap();
        let map_info = self.map_info.lock().await;
        let data_delete_locked = map_info.get(&id_delete).unwrap();
        let mut data_delete = data_delete_locked.lock().await;
        assert!(data_delete.ref_count > 0);
        data_delete.ref_count -= 1;
        self.new.lock().await[bucket][position] = None;
        if data_delete.ref_count == 0 {
            self.delete(id_delete).await;
        }
    }

    pub async fn delete(&self, id: u32) {
        //TODO finish unwraps
        assert!(self.map_info.lock().await.contains_key(&id));
        let map_info = self.map_info.lock().await;
        let data_locked = map_info.get(&id).unwrap();

        let data = data_locked.lock().await;
        //Panic if trying to delete an address inside of tried.
        assert!(!data.in_tried);
        //Panic if data is still referenced in the maps.
        assert!(data.ref_count == 0);

        self.swap_random(data.random_position, self.random.lock().await.len() - 1)
            .await;
        //TODO double check that this works.
        self.random
            .lock()
            .await
            .truncate(self.random.lock().await.len() - 1);
        self.map_address.lock().await.remove_entry(&data.address);
        self.map_info.lock().await.remove_entry(&id);
    }

    pub async fn swap_random(&self, position: usize, position_2: usize) {
        if position == position_2 {
            return;
        }

        assert!(
            position < self.random.lock().await.len()
                && position_2 < self.random.lock().await.len()
        );

        let id = self.random.lock().await[position];
        let id2 = self.random.lock().await[position_2];

        //TODO can be re-written with Matches I believe.
        assert!(self.map_info.lock().await.contains_key(&id));
        assert!(self.map_info.lock().await.contains_key(&id2));

        let map_info = self.map_info.lock().await;

        map_info.get(&id).unwrap().lock().await.random_position = position_2;
        map_info.get(&id2).unwrap().lock().await.random_position = position;

        self.random.lock().await[position] = id2;
        self.random.lock().await[position_2] = id;
    }

    //This returns a reference directly inside of the hash map. Not sure if that's the
    //functionality we want from this function, so come back to this.
    pub async fn select(&self, new_only: bool) -> Option<Arc<Mutex<PeerData>>> {
        //If the store is empty, return none.
        if self.size() == 0 {
            return None;
        };

        //If we only want new addresses, and the new addresses are empty, return none.
        if new_only && *self.new_count.lock().await == 0 {
            return None;
        }

        if !new_only
            && *self.tried_count.lock().await > 0
            && (*self.new_count.lock().await == 0 || !random(1) == 0)
        {
            let mut chance = 1.0;
            loop {
                //Make a random usize function. so that we don't have to coax all these values.
                //TODO don't coax, use usize everywhere.
                let mut bucket = random(TRIED_BUCKET_COUNT as u32);
                let mut bucket_position = random(BUCKET_SIZE as u32);

                while self.tried.lock().await[bucket as usize][bucket_position as usize].is_none() {
                    bucket = (bucket + random(TRIED_BUCKET_COUNT_LOG2 as u32))
                        % TRIED_BUCKET_COUNT as u32;
                    bucket_position =
                        (bucket_position + random(BUCKET_SIZE_LOG2)) % BUCKET_SIZE as u32;
                }

                let id = self.tried.lock().await[bucket as usize][bucket_position as usize];

                let map_info = self.map_info.lock().await;

                let info_locked = map_info.get(&id.unwrap()).unwrap();

                let info = info_locked.lock().await;

                //TODO cleaner way of doing this, probably with map error.
                //if info.is_none() {
                //    //throw error
                //}

                //TODO clean up all the coaxings.

                if (random(30) as f64) < chance * info.get_chance() * ((1 << 30) as f64) {
                    Some(info_locked.clone());
                }

                chance *= 1.2;
            }
        }

        None
    }

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

//fn make_tried(
//    tried: &mut [[Option<u32>; TRIED_BUCKET_COUNT]; BUCKET_SIZE],
//    new: &mut [[Option<u32>; NEW_BUCKET_COUNT]; BUCKET_SIZE],
//    tried_count: &mut u32,
//    new_count: &mut u32,
//    data: &mut PeerData,
//    id: &u32,
//    key: &Uint256,
//) {
//    for i in 0..NEW_BUCKET_COUNT {
//        let position = data.get_bucket_position(*key, true, i);
//        if let Some(old_id) = new[i][position] {
//            if old_id == *id {
//                new[i][position] == None;
//                data.ref_count -= 1;
//            }
//        }
//    }
//    *new_count -= 1;

//    assert!(data.ref_count == 0);

//    let bucket = data.get_tried_bucket(*key);
//    let position = data.get_bucket_position(*key, false, bucket);

//    match tried[bucket][position] {
//        None => {}
//        Some(id_evict) => {
//            assert!(self.map_info.contains_key(&id_evict));
//            //TODO remove unwraps
//            let data_old = self.map_info.get_mut(&id_evict).unwrap();

//            //Remove the to-be-evicted item from the tried set.
//            data_old.in_tried = false;
//            self.tried[bucket][position] == None;
//            self.tried_count -= 1;

//            //Find which new bucket this address belongs to.
//            let new_bucket = data_old.get_new_bucket(self.key);
//            let new_position = data_old.get_bucket_position(self.key, true, new_bucket);
//            self.clear_new(new_bucket, new_position);
//            assert!(self.new[new_bucket][new_position].is_none());

//            data_old.ref_count = 1;
//            self.new[new_bucket][new_position] = Some(id_evict);
//            self.new_count += 1;
//        }
//    }

//    assert!(self.tried[bucket][position] == None);

//    self.tried[bucket][position] = Some(id.clone());
//    self.tried_count += 1;
//    data.in_tried = true;
//}
