// use crypto::sha2::Sha256;
use cryptoxide::blake2b::Blake2b;
use cryptoxide::digest::Digest;
use encodings::hex::{FromHex, FromHexError, ToHex};
use extended_primitives::{Buffer, Hash};
use handshake_encoding::{Decodable, DecodingError, Encodable};
use handshake_protocol::consensus::consensus_verify_pow;
use sha3::{Digest as _Digest, Sha3_256};

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
    pub fn padding(&self, size: usize) -> Buffer {
        let mut padding = Vec::with_capacity(size);
        let prev_block = self.prev_block.to_array();
        let tree_root = self.tree_root.to_array();

        for i in 0..size {
            padding.push(prev_block[i % 32] ^ tree_root[i % 32]);
        }

        Buffer::from(padding)
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

impl ToHex for BlockHeader {
    fn to_hex(&self) -> String {
        self.encode().to_hex()
    }
}

impl FromHex for BlockHeader {
    type Error = DecodingError;

    fn from_hex<T: AsRef<[u8]>>(hex: T) -> std::result::Result<Self, Self::Error> {
        BlockHeader::decode(&mut Buffer::from_hex(hex)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_block_header_hex_default() {
        let block_header = BlockHeader::default();

         let hex = block_header.to_hex();
         // 212 bytes, multiply by 2 since building a String from 0s
         let expect = String::from_utf8(vec![b'0'; 212 * 2]).unwrap();
         assert_eq!(hex, expect)
    }

    #[test]
    fn test_block_header_hex() {
        let block_header = BlockHeader {
            version: 0,
            prev_block: Default::default(),
            merkle_root: Default::default(),
            witness_root: Default::default(),
            tree_root: Default::default(),
            reserved_root: Default::default(),
            time: 0,
            bits: 0,
            extra_nonce: Default::default(),
            nonce: 1,
            mask: Default::default()
        };

        let hex = block_header.to_hex();
        let expect = format!("{}{}", "01", String::from_utf8(vec![b'0'; 211 * 2]).unwrap());
        assert_eq!(hex, expect);
    }

    #[test]
    fn test_preheader_serialization() {
        let block_header = BlockHeader {
            version: 0,
            prev_block: Hash::from_hex(
                "00000000000057919601ead28513e11afb2cb5d6b4f9ebb0e2a3eeae353d21ef",
            )
            .unwrap(),
            merkle_root: Hash::from_hex(
                "a25e36543911eb25fb2f9c0187261ebce7bf71229aac631d74535fcd67504463",
            )
            .unwrap(),
            witness_root: Hash::from_hex(
                "fb442499ab4d7dc32240af3543194d9d4508ed71a461c5a37e902b7c6c626192",
            )
            .unwrap(),
            tree_root: Hash::from_hex(
                "0000000000000000000000000000000000000000000000000000000000000000",
            )
            .unwrap(),
            reserved_root: Hash::from_hex(
                "0000000000000000000000000000000000000000000000000000000000000000",
            )
            .unwrap(),
            extra_nonce: Buffer::from_hex("27000000000000000000000098f5d2f557a4f9a23d4dcc25")
                .unwrap(),
            mask: Hash::from_hex(
                "00000000000000858c7deed002d37f20cf44bebc2ae8abbf56a0fcae340f4ba7",
            )
            .unwrap(),
            time: 1580832487,
            bits: 453068266,
            nonce: 102224329,
        };

        let padding = block_header.padding(32);
        let expected_padding = Buffer::from_hex("00000000000057919601ead28513e11afb2cb5d6b4f9ebb0e2a3eeae353d21ef").unwrap();
        assert_eq!(padding, expected_padding);


        let expected_subhead = Buffer::from_hex("27000000000000000000000098f5d2f557a4f9a23d4dcc250000000000000000000000000000000000000000000000000000000000000000fb442499ab4d7dc32240af3543194d9d4508ed71a461c5a37e902b7c6c626192a25e36543911eb25fb2f9c0187261ebce7bf71229aac631d74535fcd6750446300000000ea45011b").unwrap();

        let subhead = block_header.to_subhead();
        assert_eq!(subhead, expected_subhead);

        let expected_prehead = Buffer::from_hex("c9d11706e796395e0000000000000000000057919601ead28513e11afb2cb5d600000000000057919601ead28513e11afb2cb5d6b4f9ebb0e2a3eeae353d21ef0000000000000000000000000000000000000000000000000000000000000000888d23ddee49560b017d675a21c6adc5de1a08f45c30dbc94cbbb8168faab495").unwrap();

        let prehead = block_header.to_prehead();
        assert_eq!(prehead, expected_prehead);
    }

    #[test]
    fn test_check_header_pow() {
        let block_header = BlockHeader {
            version: 0,
            prev_block: Hash::from_hex(
                "00000000000057919601ead28513e11afb2cb5d6b4f9ebb0e2a3eeae353d21ef",
            )
            .unwrap(),
            merkle_root: Hash::from_hex(
                "a25e36543911eb25fb2f9c0187261ebce7bf71229aac631d74535fcd67504463",
            )
            .unwrap(),
            witness_root: Hash::from_hex(
                "fb442499ab4d7dc32240af3543194d9d4508ed71a461c5a37e902b7c6c626192",
            )
            .unwrap(),
            tree_root: Hash::from_hex(
                "0000000000000000000000000000000000000000000000000000000000000000",
            )
            .unwrap(),
            reserved_root: Hash::from_hex(
                "0000000000000000000000000000000000000000000000000000000000000000",
            )
            .unwrap(),
            extra_nonce: Buffer::from_hex("27000000000000000000000098f5d2f557a4f9a23d4dcc25")
                .unwrap(),
            mask: Hash::from_hex(
                "00000000000000858c7deed002d37f20cf44bebc2ae8abbf56a0fcae340f4ba7",
            )
            .unwrap(),
            time: 1580832487,
            bits: 453068266,
            nonce: 102224329,
        };

        let share_hash = block_header.share_hash();
        let expected_share_hash = Hash::from_hex("000000000000a250786f48a91224e9a8b1b12933466511959572860ab5d77fff").unwrap();
        assert_eq!(expected_share_hash, share_hash);

        let pow = block_header.verify_pow();
        assert!(pow);
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
