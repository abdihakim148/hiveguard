use super::{Id, Either, Email, Phone};
use serde::{Serialize, Deserialize};
use chrono::{Utc, DateTime};


#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Verification<ID = Id> {
    pub owner_contact: Either<Email, Phone>,
    pub id: ID,
    pub code: u32,
    pub expires: DateTime<Utc>,
}