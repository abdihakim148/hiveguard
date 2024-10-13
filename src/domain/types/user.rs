use serde::{Deserialize, Serialize};
use bson::oid::ObjectId;
use super::Email;

/// A struct representing a user.
#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq)]
pub struct User {
    /// The unique identifier for the user.
    #[serde(default)]
    pub id: ObjectId,
    /// The username of the user.
    pub username: String,
    /// The first name of the user.
    pub first_name: String,
    /// The last name of the user.
    pub last_name: String,
    /// The email address of the user.
    pub email: Email,
    /// The password of the user.
    #[serde(skip_serializing_if = "is_default")]
    pub password: String,
}


fn is_default<T: Default + PartialEq>(value: &T) -> bool {
    value == &T::default()
}