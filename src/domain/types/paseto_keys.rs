use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct PasetoKeys {
    pub private_key: [u8; 32],
    pub public_key: [u8; 32],
    pub prev_public_key: Option<[u8; 32]>,
    pub created_time: DateTime<Utc>,
    pub expires: DateTime<Utc>
}