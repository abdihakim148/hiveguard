use bson::oid::ObjectId;
use serde::{Serialize, Deserialize};

/// A struct representing a member with roles in an organisation.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Member {
    /// The unique identifier for the user.
    pub id: ObjectId,
    /// The title of the member.
    pub title: String,
    /// The list of role IDs associated with the member.
    pub roles: Vec<ObjectId>,
}
