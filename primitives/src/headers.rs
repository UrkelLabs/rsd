// use crypto::sha2::Sha256;

use cryptoxide::blake2b::Blake2b;
use cryptoxide::digest::Digest;
use sha3::{Digest as _Digest, Sha3_256};
use sp800_185::KMac;

use extended_primitives::{Buffer, Hash, Uint256};
use handshake_encoding::{Decodable, DecodingError, Encodable};
use handshake_protocol::consensus::consensus_verify_pow;

//@todo FromHex and ToHex for this - pull from our encodings library.

/// A block header, which contains all the block's information except
/// the actual transactions
// #[derive(Copy, PartialEq, Eq, Clone, Debug)]
#[derive(Debug, Default, PartialEq, Clone)]
pub struct BlockHeader {
    /// The protocol version.
    pub version: u32,
    /// Reference to the previous block in the chain
    pub prev_block: Hash,
    /// The root hash of the merkle tree of transactions in the block
    pub merkle_root: Hash,

    pub witness_root: Hash,
    /// The root hash of the Urkel Tree of name states in the block
    pub tree_root: Hash,
    /// A root reserved for future implementation of Neutrino on the protocol level
    pub reserved_root: Hash,
    /// The timestamp of the block, as claimed by the miner
    /// //TODO change this to our internal time type.
    pub time: u64,
    /// The target value below which the blockhash must lie, encoded as a
    /// a float (with well-defined rounding, of course)
    /// This should probably be a Compact type - See Parity Bitcoin //TODO
    pub bits: u32,
    /// The nonce, selected to obtain a low enough blockhash
    pub nonce: u32,
    //@todo make this a type w/ a hardcoded size.
    pub extra_nonce: Buffer,
    ///A value used to mask potential blocks from miners. Defeating the block withholding attack
    pub mask: Hash,
}

impl BlockHeader {
    pub fn hash(&self) -> Hash {
        self.pow_hash()
    }

    ///Retrieve deterministically random padding.
    //@todo I don't actually this is correct. Let's come back to it later.
    pub fn padding(&self, size: usize) -> Vec<u8> {
        let mut padding = Vec::new();
        for i in 0..size {
            let padding_byte =
                self.prev_block.to_array()[i % 32] ^ self.tree_root.to_array()[i % 32];
            padding.push(padding_byte);
        }

        padding.to_vec()
    }

    /// The subheader contains miner-mutable and less essential data (that is,
    /// less essential for SPV resolvers). It is exactly one blake2b block (128 bytes).
    pub fn to_subhead(&self) -> Buffer {
        let mut buffer = Buffer::new();

        buffer.write_bytes(&self.extra_nonce);
        buffer.write_hash(self.reserved_root);
        buffer.write_hash(self.witness_root);
        buffer.write_hash(self.merkle_root);
        buffer.write_u32(self.version);
        buffer.write_u32(self.bits);

        buffer
    }

    // ===== Hash Functions ===== //

    /// Compute the subheader hash.
    pub fn sub_hash(&self) -> Hash {
        let mut sh = Blake2b::new(32);
        let mut output = [0; 32];
        sh.input(&self.to_subhead());
        sh.result(&mut output);

        Hash::from(output)
    }

    ///Compute xor bytes hash.
    pub fn mask_hash(&self) -> Hash {
        let mut sh = Blake2b::new(32);
        let mut output = [0; 32];
        sh.input(&self.prev_block.to_array());
        sh.input(&self.mask.to_array());
        sh.result(&mut output);

        Hash::from(output)
    }

    pub fn commit_hash(&self) -> Hash {
        let mut sh = Blake2b::new(32);
        let mut output = [0; 32];
        sh.input(&self.sub_hash().to_array());
        sh.input(&self.mask_hash().to_array());
        sh.result(&mut output);

        Hash::from(output)
    }

    //Writes everything but the nonce to a buffer.
    pub fn to_prehead(&self) -> Buffer {
        let mut buffer = Buffer::new();

        buffer.write_u32(self.nonce);
        buffer.write_u64(self.time);
        buffer.write_bytes(&self.padding(20));
        buffer.write_hash(self.prev_block);
        buffer.write_hash(self.tree_root);
        buffer.write_hash(self.commit_hash());

        buffer
    }

    pub fn share_hash(&self) -> Hash {
        let data = self.to_prehead();

        // 128 bytes (output as BLAKE2b-512).
        let mut sh = Blake2b::new(64);
        let mut left = [0; 64];
        sh.input(&data);
        sh.result(&mut left);

        // 128 + 8 = 136 bytes.
        let mut sh = Sha3_256::new();
        sh.input(&data);
        sh.input(self.padding(8));
        let right = sh.result();

        // 64 + 32 + 32 = 128 bytes.
        let mut sh = Blake2b::new(32);
        let mut output = [0; 32];
        sh.input(&left);
        sh.input(&self.padding(32));
        sh.input(&right);
        sh.result(&mut output);

        Hash::from(output)
    }

    pub fn pow_hash(&self) -> Hash {
        let hash = self.share_hash();
        let mask = self.mask.to_array();

        for (i, hash_byte) in hash.to_array().iter_mut().enumerate() {
            //@smells
            *hash_byte ^= mask[i];
        }

        hash
    }

    //Wrapper function for all the verification on the headers
    pub fn verify(&self) -> bool {
        //As of right now headers can just check pow, so we'll simply return that in place of this
        //function.
        self.verify_pow()
    }

    pub fn verify_pow(&self) -> bool {
        //Pass to consensus code.
        consensus_verify_pow(&self.hash(), self.bits)
    }

    //Just use fromHex, toHex here. Today let's get to this.
    //pub fn as_hex(&self) -> String {
    //    //Use prehead here.
    //    let mut buffer = Buffer::new();

    //    buffer.write_u32(self.version);
    //    buffer.write_hash(self.prev_blockhash);
    //    buffer.write_hash(self.merkle_root);
    //    buffer.write_hash(self.witness_root);
    //    buffer.write_hash(self.tree_root);
    //    buffer.write_hash(self.filter_root);
    //    buffer.write_hash(self.reserved_root);
    //    buffer.write_u64(self.time);
    //    //This will switch to write_compact when we convert the type TODO
    //    buffer.write_u32(self.bits);
    //    // buffer.write_u64(self.nonce as u64);
    //    //Think we might want to change this to write Bytes or write Buffer.
    //    //Because nonce is not *technically* a hash
    //    buffer.write_u256(self.nonce);

    //    buffer.to_hex()
    //}
}

impl Encodable for BlockHeader {
    fn size(&self) -> usize {
        //Put this into consensus TODO
        236
    }

    fn encode(&self) -> Buffer {
        let mut buffer = Buffer::new();

        //Preheader
        buffer.write_u32(self.nonce);
        buffer.write_u64(self.time);
        buffer.write_hash(self.prev_block);
        buffer.write_hash(self.tree_root);

        //Subheader
        buffer.write_bytes(&self.extra_nonce);
        buffer.write_hash(self.reserved_root);
        buffer.write_hash(self.witness_root);
        buffer.write_hash(self.merkle_root);
        buffer.write_u32(self.version);
        buffer.write_u32(self.bits);

        //Mask
        buffer.write_bytes(&self.mask.to_array());

        buffer
    }
}

impl Decodable for BlockHeader {
    type Err = DecodingError;

    fn decode(buffer: &mut Buffer) -> Result<Self, Self::Err> {
        //Preheader
        let nonce = buffer.read_u32()?;
        let time = buffer.read_u64()?;
        let prev_block = buffer.read_hash()?;
        let tree_root = buffer.read_hash()?;

        //Subheader
        //@todo put this into consensus NONCE_SIZE
        let extra_nonce = buffer.read_bytes(24)?;
        let reserved_root = buffer.read_hash()?;
        let witness_root = buffer.read_hash()?;
        let merkle_root = buffer.read_hash()?;
        let version = buffer.read_u32()?;
        let bits = buffer.read_u32()?;

        //Mask
        //@todo figure out the right type for mask.
        let mask = buffer.read_hash()?;

        Ok(BlockHeader {
            version,
            prev_block,
            merkle_root,
            witness_root,
            tree_root,
            reserved_root,
            extra_nonce: Buffer::from(extra_nonce),
            time,
            bits,
            nonce,
            mask,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hex::FromHex;

    // #[test]
    // fn test_block_header_hex_default() {
    //     let block_header = BlockHeader::default();

    //     // let hex = block_header.as_hex();

    //     assert_eq!(hex, "000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000")
    // }

    // #[test]
    // fn test_block_header_hex() {
    //     let block_header = BlockHeader {
    //         version: 1,
    //         prev_blockhash: Default::default(),
    //         merkle_root: Default::default(),
    //         witness_root: Default::default(),
    //         tree_root: Default::default(),
    //         filter_root: Default::default(),
    //         reserved_root: Default::default(),
    //         time: 2,
    //         bits: 3,
    //         nonce: Default::default(),
    //     };

    //     // let hex = block_header.as_hex();

    //     assert_eq!(hex, "010000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000200000000000000030000000000000000000000000000000000000000000000000000000000000000000000")
    // }

    #[test]
    fn test_check_header_pow() {
        //We will need to pass a legit block for this test TODO
        let block_header = BlockHeader {
            version: 1,
            prev_block: Hash::from(
                "0101010101010101010101010101010101010101010101010101010101010101",
            ),
            merkle_root: Hash::from(
                "0101010101010101010101010101010101010101010101010101010101010101",
            ),
            witness_root: Hash::from(
                "0202020202020202020202020202020202020202020202020202020202020202",
            ),
            tree_root: Hash::from(
                "0303030303030303030303030303030303030303030303030303030303030303",
            ),
            reserved_root: Hash::from(
                "0404040404040404040404040404040404040404040404040404040404040404",
            ),
            extra_nonce: Buffer::from_hex("050505050505050505050505050505050505050505050505")
                .unwrap(),
            mask: Hash::from("0606060606060606060606060606060606060606060606060606060606060606"),
            time: 0,
            bits: 0,
            nonce: 0,
        };

        dbg!(block_header.share_hash());

        // let pow = block_header.verify_pow();

        // assert!(pow);
    }

    // #[test]
    // fn test_headers_verify_pow_2() {
    //let nonce_bytes =
    //    hex::decode("9e45f30200000000000000000000000000000000000000000000000000000000")
    //        .unwrap();

    //let block_header = BlockHeader {
    //    version: 0,
    //    prev_blockhash: Hash::from(
    //        "3bf6e7d1ac019692790cf617ec155dd6254fb010468fa5d1b91979cb7362d247",
    //    ),
    //    merkle_root: Hash::from(
    //        "80f80dc13cd18c520f5322b2b8fbbad5b96f45945331eff3c8c032137c80d274",
    //    ),
    //    witness_root: Hash::from(
    //        "44edb180cd43fca87c1c692c947381e4476c67d673b8f086a0cc783f43be379f",
    //    ),
    //    tree_root: Hash::from(
    //        "fc1bda0f826d2bb09536d42fd8beb327ac0c8c60322ce78bbfc2af2cbec4cf4d",
    //    ),
    //    filter_root: Hash::from(
    //        "0000000000000000000000000000000000000000000000000000000000000000",
    //    ),
    //    reserved_root: Hash::from(
    //        "0000000000000000000000000000000000000000000000000000000000000000",
    //    ),
    //    time: 1558043457,
    //    bits: 489684992,
    //    nonce: Uint256::from_big_endian(&nonce_bytes),
    //};

    //let pow = block_header.verify_pow();

    //assert!(pow);
    //}

    //#[test]
    //fn test_block_header_hash() {
    ////Test mainnet genesis block
    //let block_header = BlockHeader {
    //    version: 0,
    //    prev_blockhash: Hash::from(
    //        "0000000000000000000000000000000000000000000000000000000000000000",
    //    ),
    //    merkle_root: Hash::from(
    //        "8e4c9756fef2ad10375f360e0560fcc7587eb5223ddf8cd7c7e06e60a1140b15",
    //    ),
    //    witness_root: Hash::from(
    //        "7c7c2818c605a97178460aad4890df2afcca962cbcb639b812db0af839949798",
    //    ),
    //    tree_root: Hash::from(
    //        "0000000000000000000000000000000000000000000000000000000000000000",
    //    ),
    //    filter_root: Hash::from(
    //        "0000000000000000000000000000000000000000000000000000000000000000",
    //    ),
    //    reserved_root: Hash::from(
    //        "0000000000000000000000000000000000000000000000000000000000000000",
    //    ),

    //    time: 1554268735,
    //    bits: 486604799,
    //    nonce: Uint256::default(),
    //};

    //// let hex = block_header.as_hex();

    //assert_eq!(hex, "0000000000000000000000000000000000000000000000000000000000000000000000008e4c9756fef2ad10375f360e0560fcc7587eb5223ddf8cd7c7e06e60a1140b157c7c2818c605a97178460aad4890df2afcca962cbcb639b812db0af8399497980000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000003f42a45c00000000ffff001d0000000000000000000000000000000000000000000000000000000000000000");

    //let hash = block_header.hash();

    //assert_eq!(
    //    &hash.to_string(),
    //    "b08ff0f0e33bca4cd80a7f1dda3f545a00b72a7a144b6b8d1a30150a78f7975c"
    //);
    //}
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
