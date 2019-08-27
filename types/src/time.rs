use chrono::{DateTime, Utc};
use std::cmp;
use std::ops;

//TODO I think Eq impls Partial Eq and for Ord same thing. remoe if so.
#[derive(Copy, Debug, Clone, Default, Eq, Ord)]
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
impl<T> ops::Add<T> for Time
where
    T: Into<Time>,
{
    type Output = Self;

    fn add(self, other: T) -> Self {
        Self(self.0 + other.into().0)
    }
}

impl<T> ops::Sub<T> for Time
where
    T: Into<Time>,
{
    type Output = Self;

    fn sub(self, other: T) -> Self {
        Self(self.0 - other.into().0)
    }
}

//Comparisons

impl cmp::PartialEq<u64> for Time {
    fn eq(&self, other: &u64) -> bool {
        &self.0 == other
    }
}

impl cmp::PartialEq for Time {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl cmp::PartialOrd<u64> for Time {
    fn partial_cmp(&self, other: &u64) -> Option<cmp::Ordering> {
        Some(self.0.cmp(other))
    }
}

impl cmp::PartialOrd for Time {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        Some(self.0.cmp(&other.0))
    }
}

//TODO impl From Datetime, SystemTime, Duration, and u64
