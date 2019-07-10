use chrono::{DateTime, Utc};
use std::ops;

//TODO I think Eq impls Partial Eq and for Ord same thing. remoe if so.
#[derive(PartialEq, Eq, PartialOrd, Ord, Copy, Debug, Clone, Default)]
pub struct Time(u64);

//Time in Seconds
impl Time {
    //Returns a time of 0.
    pub fn new() -> Self {
        Default::default()
    }

    pub fn now() -> Self {
        Time(Utc::now().timestamp() as u64)
    }

    pub fn to_seconds(self) -> u64 {
        self.0
    }
}

impl From<u64> for Time {
    fn from(time: u64) -> Self {
        Time(time)
    }
}

//Operators
//These can probably all be combined with generics and FROM traits.
//Add<T: Into<Time>>
impl ops::Add for Time {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self(self.0 + other.0)
    }
}

impl ops::Add<u64> for Time {
    type Output = Self;

    fn add(self, other: u64) -> Self {
        Self(self.0 + other)
    }
}

impl ops::Sub for Time {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self(self.0 - other.0)
    }
}

impl ops::Sub<u64> for Time {
    type Output = Self;

    fn sub(self, other: u64) -> Self {
        Self(self.0 - other)
    }
}

//TODO make sure we can do add, minus, etc

//TODO impl From Datetime, SystemTime, Duration, and u64
