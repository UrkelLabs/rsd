use blake2::Blake2b;

pub struct Address {
    pub version: u32,
    pub hash: Blake2b
}
