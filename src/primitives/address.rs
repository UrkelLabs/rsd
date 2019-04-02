use blake2::Blake2b;

pub struct Address {
    pub version: u32,
    pub hash: Blake2b
}

impl From<String> for Address {
    fn from(item: i32) -> Self {
        Number { value: item }
    }
}


