// use crypto::sha2::Sha256;


// /// A block header, which contains all the block's information except
// /// the actual transactions
// // #[derive(Copy, PartialEq, Eq, Clone, Debug)]
// pub struct BlockHeader {
//     /// The protocol version.
//     pub version: u32,
//     /// Reference to the previous block in the chain
//     pub prev_blockhash: Sha256,
//     /// The root hash of the merkle tree of transactions in the block
//     pub merkle_root: Sha256,
//     /// The root hash of the Urkel Tree of name states in the block
//     pub tree_root: Sha256,
//     /// The root hash of the bloom filter XXX Need more here.
//     pub filter_root: Sha256,
//     /// A root reserved for future implementation of Neutrino on the protocol level
//     pub reserved_root: Sha256,
//     /// The timestamp of the block, as claimed by the miner
//     pub time: u32,
//     /// The target value below which the blockhash must lie, encoded as a
//     /// a float (with well-defined rounding, of course)
//     pub bits: u32,
//     /// The nonce, selected to obtain a low enough blockhash
//     pub nonce: u32,
// }

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
