// use crypto::sha2::Sha256;

use cryptoxide::blake2b::Blake2b;
use cryptoxide::digest::Digest;
use sp800_185::KMac;

use handshake_protocol::consensus::consensus_verify_pow;
use handshake_types::{Buffer, Hash, Uint256};

/// A block header, which contains all the block's information except
/// the actual transactions
// #[derive(Copy, PartialEq, Eq, Clone, Debug)]
#[derive(Debug, Default)]
pub struct BlockHeader {
    /// The protocol version.
    pub version: u32,
    /// Reference to the previous block in the chain
    pub prev_blockhash: Hash,
    /// The root hash of the merkle tree of transactions in the block
    pub merkle_root: Hash,

    pub witness_root: Hash,
    /// The root hash of the Urkel Tree of name states in the block
    pub tree_root: Hash,
    /// The root hash of the bloom filter XXX Need more here.
    pub filter_root: Hash,
    /// A root reserved for future implementation of Neutrino on the protocol level
    pub reserved_root: Hash,
    /// The timestamp of the block, as claimed by the miner
    pub time: u64,
    /// The target value below which the blockhash must lie, encoded as a
    /// a float (with well-defined rounding, of course)
    /// This should probably be a Compact type - See Parity Bitcoin //TODO
    pub bits: u32,
    /// The nonce, selected to obtain a low enough blockhash
    //Change this to Buffer, or Bytes some kind of raw type. - let's see what the output of our kmac function is.
    //256 uint
    pub nonce: Uint256,
}

impl BlockHeader {
    pub fn hash(&self) -> Hash {
        let mut hasher = Blake2b::new(32);
        //I think can just pass without decoding... TODO
        hasher.input(&hex::decode(self.as_hex()).unwrap());
        // let hash = hasher.finalize();
        let mut out = [0; 32];

        hasher.result(&mut out);

        Hash::from(hex::encode(out))
    }

    //Writes everything but the nonce to a buffer.
    pub fn to_prehead(&self) -> Buffer {
        let mut buffer = Buffer::new();

        buffer.write_u32(self.version);
        buffer.write_hash(self.prev_blockhash);
        buffer.write_hash(self.merkle_root);
        buffer.write_hash(self.witness_root);
        buffer.write_hash(self.tree_root);
        buffer.write_hash(self.filter_root);
        buffer.write_hash(self.reserved_root);
        buffer.write_u64(self.time);
        buffer.write_u32(self.bits);

        buffer
    }

    //Wrapper function for all the verification on the headers
    pub fn verify(&self) -> bool {
        //As of right now headers can just check pow, so we'll simply return that in place of this
        //function.
        self.verify_pow()
    }

    pub fn verify_pow(&self) -> bool {
        let data = self.to_prehead();

        let mut key = [0; 32];

        let mut kmac = KMac::new_kmac256(&self.nonce.to_le_bytes(), &[]);
        kmac.update(&data);
        kmac.finalize(&mut key);

        let mut hasher = Blake2b::new_keyed(32, &key);
        hasher.input(&data);

        let mut hash = [0; 32];

        hasher.result(&mut hash);

        dbg!(&hash);

        //Pass to consensus code.
        consensus_verify_pow(&Hash::from(hash), self.bits)
    }

    pub fn as_hex(&self) -> String {
        //Use prehead here.
        let mut buffer = Buffer::new();

        buffer.write_u32(self.version);
        buffer.write_hash(self.prev_blockhash);
        buffer.write_hash(self.merkle_root);
        buffer.write_hash(self.witness_root);
        buffer.write_hash(self.tree_root);
        buffer.write_hash(self.filter_root);
        buffer.write_hash(self.reserved_root);
        buffer.write_u64(self.time);
        //This will switch to write_compact when we convert the type TODO
        buffer.write_u32(self.bits);
        // buffer.write_u64(self.nonce as u64);
        //Think we might want to change this to write Bytes or write Buffer.
        //Because nonce is not *technically* a hash
        buffer.write_u256(self.nonce);

        buffer.to_hex()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn test_block_header_hex_default() {
        let block_header = BlockHeader::default();

        let hex = block_header.as_hex();

        dbg!(hex);
    }

    #[test]
    fn test_block_header_hex() {
        let block_header = BlockHeader {
            version: 1,
            prev_blockhash: Default::default(),
            merkle_root: Default::default(),
            witness_root: Default::default(),
            tree_root: Default::default(),
            filter_root: Default::default(),
            reserved_root: Default::default(),
            time: 2,
            bits: 3,
            nonce: Default::default(),
        };

        let hex = block_header.as_hex();

        dbg!(hex);
    }

    #[test]
    fn test_check_header_pow() {
        //We will need to pass a legit block for this test TODO
        let block_header = BlockHeader {
            version: 1,
            prev_blockhash: Default::default(),
            merkle_root: Default::default(),
            witness_root: Default::default(),
            tree_root: Default::default(),
            filter_root: Default::default(),
            reserved_root: Default::default(),
            time: 2,
            bits: 486604799,
            nonce: Default::default(),
        };

        let pow = block_header.verify_pow();

        assert!(pow);
    }

    #[test]
    fn test_headers_verify_pow_2() {
        let nonce_bytes =
            hex::decode("9e45f30200000000000000000000000000000000000000000000000000000000")
                .unwrap();

        let block_header = BlockHeader {
            version: 0,
            prev_blockhash: Hash::from(
                "3bf6e7d1ac019692790cf617ec155dd6254fb010468fa5d1b91979cb7362d247",
            ),
            merkle_root: Hash::from(
                "80f80dc13cd18c520f5322b2b8fbbad5b96f45945331eff3c8c032137c80d274",
            ),
            witness_root: Hash::from(
                "44edb180cd43fca87c1c692c947381e4476c67d673b8f086a0cc783f43be379f",
            ),
            tree_root: Hash::from(
                "fc1bda0f826d2bb09536d42fd8beb327ac0c8c60322ce78bbfc2af2cbec4cf4d",
            ),
            filter_root: Hash::from(
                "0000000000000000000000000000000000000000000000000000000000000000",
            ),
            reserved_root: Hash::from(
                "0000000000000000000000000000000000000000000000000000000000000000",
            ),
            time: 1558043457,
            bits: 489684992,
            nonce: Uint256::from_big_endian(&nonce_bytes),
        };

        let pow = block_header.verify_pow();

        assert!(pow);
    }

    #[test]
    fn test_block_header_hash() {
        //Test mainnet genesis block
        let block_header = BlockHeader {
            version: 0,
            prev_blockhash: Hash::from(
                "0000000000000000000000000000000000000000000000000000000000000000",
            ),
            merkle_root: Hash::from(
                "8e4c9756fef2ad10375f360e0560fcc7587eb5223ddf8cd7c7e06e60a1140b15",
            ),
            witness_root: Hash::from(
                "7c7c2818c605a97178460aad4890df2afcca962cbcb639b812db0af839949798",
            ),
            tree_root: Hash::from(
                "0000000000000000000000000000000000000000000000000000000000000000",
            ),
            filter_root: Hash::from(
                "0000000000000000000000000000000000000000000000000000000000000000",
            ),
            reserved_root: Hash::from(
                "0000000000000000000000000000000000000000000000000000000000000000",
            ),

            time: 1554268735,
            bits: 486604799,
            nonce: Uint256::default(),
        };

        let hex = block_header.as_hex();

        assert_eq!(hex, "0000000000000000000000000000000000000000000000000000000000000000000000008e4c9756fef2ad10375f360e0560fcc7587eb5223ddf8cd7c7e06e60a1140b157c7c2818c605a97178460aad4890df2afcca962cbcb639b812db0af8399497980000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000003f42a45c00000000ffff001d0000000000000000000000000000000000000000000000000000000000000000");

        let hash = block_header.hash();

        assert_eq!(
            &hash.to_string(),
            "b08ff0f0e33bca4cd80a7f1dda3f545a00b72a7a144b6b8d1a30150a78f7975c"
        );
    }

}

// /// A block header with txcount attached, which is given in the `headers`
// /// network message.
// // #[derive(PartialEq, Eq, Clone, Debug)]
// pub struct LoneBlockHeader {
//         /// The actual block header
// 	pub header: BlockHeader,
// 	// /// The number of transactions in the block. This will always be zero
// 	// /// when the LoneBlockHeader is returned as part of a `headers` message.
// 	pub tx_count: u32
// }

// impl BlockHeader {
// /// Computes the target [0, T] that a blockhash must land in to be valid
//     pub fn target(&self) -> Uint256 {
//         // This is a floating-point "compact" encoding originally used by
//         // OpenSSL, which satoshi put into consensus code, so we're stuck
//         // with it. The exponent needs to have 3 subtracted from it, hence
//         // this goofy decoding code:
//         let (mant, expt) = {
//             let unshifted_expt = self.bits >> 24;
//             if unshifted_expt <= 3 {
//                 ((self.bits & 0xFFFFFF) >> (8 * (3 - unshifted_expt as usize)), 0)
//             } else {
//                 (self.bits & 0xFFFFFF, 8 * ((self.bits >> 24) - 3))
//             }
//         };

//         // The mantissa is signed but may not be negative
//         if mant > 0x7FFFFF {
//             Default::default()
//         } else {
//             Uint256::from_u64(mant as u64).unwrap() << (expt as usize)
//         }
//     }
// }
