use crate::Hash;

#[derive(PartialEq, Eq, Clone, Debug)]
pub struct Address {
    pub version: u32,
    pub hash: Hash,
}

impl Address {
    //TODO
    // pub fn is_null(&self) -> bool {
    //     self.hash.is_null()
    // }

    pub fn is_null_data(&self) -> bool {
        self.version == 31
    }

    pub fn is_unspendable(&self) -> bool {
        self.is_null_data()
    }
}

// impl From<String> for Address {
//     fn from(item: i32) -> Self {
//         Address { value: item }
//     }
// }
