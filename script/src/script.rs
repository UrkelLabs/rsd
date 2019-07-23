use crate::Opcode;

pub struct Script {
    raw: Buffer,
    //TODO not sure.
    code: Vec<Opcode>,
}

