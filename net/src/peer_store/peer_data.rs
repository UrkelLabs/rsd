use crate::peer_store::common::{
    BUCKET_SIZE, HORIZON_DAYS, MAX_FAILURES, MIN_FAIL_DAYS, NEW_BUCKETS_PER_SOURCE_GROUP,
    NEW_BUCKET_COUNT, RETRIES, TRIED_BUCKETS_PER_GROUP, TRIED_BUCKET_COUNT,
};
use crate::NetAddress;
use extended_primitives::{uint256::Uint256, Buffer};
use fasthash::murmur3;
use handshake_types::Time;

pub struct PeerData {
    //Possibly change these to u64, but Datetime should work as well
    //Let's change this to the internal time type though.
    pub last_counted_try: Time,
    pub last_try: Time,
    pub address: NetAddress,

    // These should probably be protected.
    source: NetAddress,
    pub(crate) last_success: Time,
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
    pub fn get_tried_bucket(&self, key: Uint256) -> usize {
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

        let hash = murmur3::hash32(hash_data) as usize;

        hash % TRIED_BUCKET_COUNT
    }

    //Calculate which "new" bucket this entry belows, dependent on the source.
    //None will calculate with the default source.
    pub fn get_new_bucket(&self, key: Uint256, src: NetAddress) -> usize {
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

        let hash = murmur3::hash32(hash_data) as usize;

        hash % NEW_BUCKET_COUNT
    }

    /// Calculate in which position of a bucket to store this entry.
    /// Returns the index of the bucket.
    pub fn get_bucket_position(&self, key: Uint256, new: bool, bucket: usize) -> usize {
        let mut hash_data = Buffer::new();
        hash_data.write_u256(key);
        if new {
            hash_data.write_str("N");
        } else {
            hash_data.write_str("K");
        }
        //TODO we might want to build a write_usize function for Buffer, and then not convert this.
        hash_data.write_u32(bucket as u32);
        hash_data.extend_from_slice(self.address.get_unique_key().as_ref());

        let hash = murmur3::hash32(hash_data) as usize;

        hash % BUCKET_SIZE
    }

    //TODO I think this should take a time in.
    //Possibly take in adjusted time.
    pub fn is_terrible(&self) -> bool {
        let now = Time::now();
        // never remove things tried in the last minute
        if self.last_try != 0 && self.last_try >= now - 60 {
            return false;
        }

        //If the address came too far into the future, it is terrible
        if self.address.time > now + 600 {
            return true;
        }

        // Address not seen in recent history (beyond the horizaon days)
        if self.address.time == 0
            || (now - self.address.time) > (HORIZON_DAYS * 24 * 60 * 60) as u64
        {
            return true;
        }

        //Tried more than the RETRIES number, and still hasn't be successful
        if self.last_success == 0 && self.attempts >= RETRIES {
            return true;
        }

        //Hit the max number of failures in the last MIN_FAIL_DAYS
        if (now - self.last_success) > (MIN_FAIL_DAYS * 24 * 60 * 60) as u64
            && self.attempts >= MAX_FAILURES
        {
            return true;
        }

        false
    }

    pub fn get_chance(&self) -> f64 {
        let mut chance = 1.0;
        let since_last_try = std::cmp::max(Time::now() - self.last_try, Time::new());

        if since_last_try < 60 * 10 {
            chance *= 0.01;
        }

        chance *= 0.66f64.powi(std::cmp::min(self.attempts as i32, 8));

        chance
    }

    pub(crate) fn is_valid(&self) -> bool {
        self.address.is_valid()
    }
}

//IMPL FROM PEER
//PeerData::from(Peer);
//
