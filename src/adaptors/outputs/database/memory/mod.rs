mod tables;

use crate::ports::outputs::database::{Database, Result};
pub use tables::*;
use crate::ports::outputs::database::Table;

/// A struct representing an in-memory database.
/// A struct representing an in-memory database.
pub struct Memory {
    users: Users,
}

impl Database for Memory {
    type Users = Users;

    async fn new<T>(_args: T) -> Result<Self> {
        let users = Users::new().await?;
        Ok(Memory { users })
    }

    async fn users<'a>(&'a self) -> &'a Self::Users {
        &self.users
    }
}
