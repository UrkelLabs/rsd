use crate::types::services::Services;
use crate::NetAddress;
use chrono::{DateTime, Utc};

pub struct PeerData {
    pub address: NetAddress,
    pub source: NetAddress,
    pub services: Services,
    pub attempts: u32,
    pub last_success: DateTime<Utc>,
    pub last_attempt: DateTime<Utc>,
}
