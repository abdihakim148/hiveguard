use serde::{Deserialize, Serialize};
use crate::domain::services::Crud;
use bson::oid::ObjectId;

/// A struct representing a user.
#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq)]
pub struct User {
    /// The unique identifier for the user.
    pub id: ObjectId,
    /// The username of the user.
    pub username: String,
    /// The first name of the user.
    pub first_name: String,
    /// The last name of the user.
    pub last_name: String,
    /// The email address of the user.
    pub email: String,
    /// The password of the user.
    pub password: String,
}



impl Crud for User {
    type Id = ObjectId;
}