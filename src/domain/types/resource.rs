use bson::oid::ObjectId;
use serde::{Serialize, Deserialize};

/// A struct representing a resource.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Resource {
    /// The unique identifier for the resource.
    pub id: ObjectId,
    /// The name of the resource.
    pub name: String,
    /// The URL of the resource, if available.
    pub url: Option<String>,
}
