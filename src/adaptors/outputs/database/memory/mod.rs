/// Module for database tables.
// mod tables;
// mod error;


use crate::ports::outputs::database::CreateItem;
use crate::domain::types::{Error, User};
use serde::{Serialize, Deserialize};

/// A struct representing an in-memory database.
/// A struct representing an in-memory database.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Memory {}


impl CreateItem<User> for Memory {
    type Error = Error;
    async fn create_item(&self, item: User) -> Result<User, Self::Error> {
        todo!()
    }
}