use crate::peer_store::common::{TRIED_BUCKETS_PER_GROUP, TRIED_BUCKET_COUNT};
use crate::NetAddress;
use chrono::{DateTime, Utc};
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
        //Hash 1
        let hash_data = Buffer::new();
        hash_data.write_u256(key);
        hash_data.extend_from_slice(self.address.get_unique_key().as_ref());
        let hash = murmur3::hash32(hash_data);

        //Hash 2
        let hash_data = Buffer::new();
        hash_data.write_u256(key);
        hash_data.append(self.address.get_group().as_mut());
        hash_data.write_u32(hash % TRIED_BUCKETS_PER_GROUP);

        let hash = murmur3::hash32(hash_data);

        hash % TRIED_BUCKET_COUNT as u32
    }

    //Calculate which "new" bucket this entry belows, dependent on the source.
    //None will calculate with the default source.
    pub fn get_new_bucket(key: Uint256, src: Option<NetAddress>) -> u32 {
        //TODO its possible we just use the internal source here instead of a param.
        unimplemented!()
    }

    /// Calculate in which position of a bucket to store this entry.
    /// Returns the index of the bucket.
    pub fn get_bucket_position(key: Uint256, new: bool, bucket: u32) -> u32 {
        unimplemented!()
    }

    //TODO I think this should take a time in.
    //Possibly take in adjusted time.
    pub fn is_terrible() -> bool {
        unimplemented!()
    }

    //TODO possibly take time in -> adjusted time. See above.
    pub fn get_chance() -> f64 {
        unimplemented!()
    }
}

//IMPL FROM PEER
//PeerData::from(Peer);
//
