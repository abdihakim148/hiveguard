use serde::{Serialize, Deserialize};
use chrono::{Utc, DateTime};
use super::Id;

// The Session struct represents a user session in the system.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Session {
    pub id: Id,
    pub user_id: Id,
    pub is_active: bool,
    pub refresh_token_id: Id,
    pub previous_refresh_token_id: Option<Id>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}