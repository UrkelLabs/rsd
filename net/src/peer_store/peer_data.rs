use crate::peer_store::common::{
    BUCKET_SIZE, MIN_FAIL_DAYS, NEW_BUCKETS_PER_SOURCE_GROUP, NEW_BUCKET_COUNT,
    TRIED_BUCKETS_PER_GROUP, TRIED_BUCKET_COUNT,
};
use crate::NetAddress;
use chrono::{DateTime, Duration, Utc};
use extended_primitives::{uint256::Uint256, Buffer};
use fasthash::murmur3;

pub struct PeerData {
    //Possibly change these to u64, but Datetime should work as well
    //Let's change this to the internal time type though.
    pub last_counted_try: DateTime<Utc>,
    pub last_try: DateTime<Utc>,
    pub address: NetAddress,

    source: NetAddress,
    last_success: DateTime<Utc>,
    attempts: u32,
    ref_count: u32,
    in_tried: bool,
    //TODO figure out what this is.
    random_position: u32,
}

//TODO impl Defaults
//TODO tests from bitcoin / test all functions.

impl PeerData {
    // pub fn new() -> Self {}

    /// Calculate which "tried" bucket this entry belongs
    pub fn get_tried_bucket(&self, key: Uint256) -> u32 {
        // Hash 1
        let mut hash_data = Buffer::new();
        hash_data.write_u256(key);
        hash_data.extend_from_slice(self.address.get_unique_key().as_ref());
        let hash = murmur3::hash32(hash_data);

        // Hash 2
        let mut hash_data = Buffer::new();
        hash_data.write_u256(key);
        hash_data.append(self.address.get_group().as_mut());
        hash_data.write_u32(hash % TRIED_BUCKETS_PER_GROUP);

        let hash = murmur3::hash32(hash_data);

        hash % TRIED_BUCKET_COUNT as u32
    }

    //Calculate which "new" bucket this entry belows, dependent on the source.
    //None will calculate with the default source.
    pub fn get_new_bucket(&self, key: Uint256, src: NetAddress) -> u32 {
        //TODO its possible we just use the internal source here instead of a param.
        let mut source_group = src.get_group();

        // Hash 1
        let mut hash_data = Buffer::new();
        hash_data.write_u256(key);
        hash_data.append(self.address.get_group().as_mut());
        hash_data.append(&mut source_group);

        let hash = murmur3::hash32(hash_data);

        // Hash 2
        let mut hash_data = Buffer::new();
        hash_data.write_u256(key);
        hash_data.append(&mut source_group);
        hash_data.write_u32(hash % NEW_BUCKETS_PER_SOURCE_GROUP);

        let hash = murmur3::hash32(hash_data);

        hash % NEW_BUCKET_COUNT as u32
    }

    /// Calculate in which position of a bucket to store this entry.
    /// Returns the index of the bucket.
    pub fn get_bucket_position(&self, key: Uint256, new: bool, bucket: u32) -> u32 {
        let mut hash_data = Buffer::new();
        hash_data.write_u256(key);
        if new {
            hash_data.write_str("N");
        } else {
            hash_data.write_str("K");
        }
        hash_data.write_u32(bucket);
        hash_data.extend_from_slice(self.address.get_unique_key().as_ref());

        let hash = murmur3::hash32(hash_data);

        hash % BUCKET_SIZE as u32
    }

    //TODO I think this should take a time in.
    //Possibly take in adjusted time.
    pub fn is_terrible(&self) -> bool {
        let now = Utc::now();
        // never remove things tried in the last minute
        // Check that last_try is not null AND last try is less than 1 min ago
        // Switch to this when we use custom date time.
        // I don't think this works, but worth a shot
        if (self.last_try >= now - Duration::seconds(60)) {
            return false;
        }

        if self.address.time > now + Duration::seconds(600) {
            return true;
        }

        //If self.address.time is null
        if self.address.time == 0 || now - self.address.time > Duration::days(HORIZON_DAYS) {
            return true;
        }

        //Isn't null
        if self.last_success == 0 && self.attempts >= RETRIES {
            return true;
        }

        if now - self.last_success > Duration::days(MIN_FAIL_DAYS) && self.attempts >= MAX_FAILURES
        {
            return true;
        }

        false
    }

    pub fn get_chance(&self) -> f64 {
        let mut chance = 1.0;
        let since_last_try = std::cmp::max(Utc::now().timestamp() - self.last_try.timestamp(), 0);

        if since_last_try < 60 * 10 {
            chance *= 0.01;
        }

        chance *= 0.66f64.powi(std::cmp::min(self.attempts as i32, 8));

        chance
    }
}

//IMPL FROM PEER
//PeerData::from(Peer);
//
