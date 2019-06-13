#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Eq, Ord)]
pub struct Difficulty {
    num: u64,
}

impl Difficulty {
    pub fn zero() -> Difficulty {
        Difficulty { num: 0 }
    }
}
