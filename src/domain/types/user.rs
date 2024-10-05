use bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq)]
pub struct User {
    pub id: ObjectId,
    pub username: String,
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub password: String,
}
