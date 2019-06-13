use crate::Outpoint;

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Input {
    pub prevout: Outpoint,
    pub sequence: u32,
    //TODO this should probably be a custom type, but can be implemented later.
    pub witness: Vec<Vec<u8>>,
}
