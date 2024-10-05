use crate::ports::output::database::Table;
use crate::domain::types::User;
use std::collections::HashMap;
use bson::oid::ObjectId;
use std::sync::RwLock;


pub struct Users {
    emails: RwLock<HashMap<String, ObjectId>>,
    users: RwLock<HashMap<ObjectId, User>>
}