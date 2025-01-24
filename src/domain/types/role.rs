use serde::{Serialize, Deserialize};
use super::{Resource, Permission};
use bson::oid::ObjectId;

/// A struct representing a role with specific permissions on resources.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Role {
    /// The unique identifier for the role.
    pub id: ObjectId,
    /// The list of resources and their associated permissions.
    pub grants: Vec<(Resource, Vec<Permission>)>,
}
