use serde::{Serialize, Deserialize};
use bson::oid::ObjectId;
use super::EmailAddress;

/// A struct representing an organisation.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Organisation {
    /// The unique identifier for the organisation.
    pub id: ObjectId,
    /// The name of the organisation.
    pub name: String,
    /// The user ids of the founders of the Organisation.
    pub founders: Vec<ObjectId>,
    /// A list of named email addresses associated with the organisation.
    pub emails: Vec<(String, EmailAddress)>,
    /// The domain of the organisation, if available.
    pub domain: Option<String>,
    /// The home URL of the organisation, if available.
    pub home: Option<String>,
    /// A list of named phone numbers associated with the organisation.
    pub phone: Vec<(String, String)>,
}
