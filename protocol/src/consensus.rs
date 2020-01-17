use extended_primitives::{Hash, Uint256};
use handshake_types::Amount;
use std::result::Result;

pub const BASE_REWARD: u32 = 2_000;

pub fn max_coin() -> Amount {
    Amount::from_doos(2_040_000_000_000_000)
}

pub fn get_reward(height: u32, interval: u32) -> Amount {
    //@todo test this at max bounds to double check these casts.
    let halvings = (height as f32 / interval as f32).floor() as u32;

    if halvings >= 53 {
        return Amount::ZERO;
    }

    let exponent = 2u32.pow(halvings);

    let amount = (BASE_REWARD as f32 / exponent as f32).floor() * 1_000f32;

    Amount::from_doos(amount as u64)
}

//Make this a compact type. TODO
//TODO needs test
pub fn consensus_verify_pow(hash: &Hash, bits: u32) -> bool {
    let result = target_from_compact_bits(bits);
    let target: Uint256;

    //TODO redo this block.
    if result.is_err() {
        return false;
    } else {
        //Not sure if this is safe, TODO
        target = result.unwrap();
    };

    //TODO implement deref for hash then we don't need to array here.
    let hash_number = Uint256::from_big_endian(&hash.to_array());

    dbg!(&hash_number);
    dbg!(&target);

    hash_number <= target
}

//Again make this a Compact type -> Maybe implement target from that function in the future.
//TODO needs test - a lot of tests
//TODO I don't like the idea of returning in error, but I can think this one over.
pub fn target_from_compact_bits(bits: u32) -> Result<Uint256, Uint256> {
    let exponent = bits >> 24;
    //TODO let's make sure these are equal
    // let negative = (bits >> 23) & 1;
    let mut mantissa = bits & 0x_7ff_fff;
    let negative = mantissa != 0 && (bits & 0x00800000) != 0;

    let result = if exponent <= 3 {
        mantissa >>= 8 * (3 - exponent as usize);
        Uint256::from(mantissa)
    } else {
        Uint256::from(mantissa) << (8 * (exponent as usize - 3))
    };

    let overflow = (mantissa != 0 && exponent > 34)
        || (mantissa > 0xff && exponent > 33)
        || (mantissa > 0xffff && exponent > 32);

    if negative || overflow {
        Err(result)
    } else {
        Ok(result)
    }
}
