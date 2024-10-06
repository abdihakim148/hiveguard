mod tables;

use crate::ports::output::database::{Database, Result};
use tables::users::Users;
use crate::ports::output::database::Table;

/// A struct representing an in-memory database.
pub struct Memory {
    users: Users,
}

impl Database for Memory {
    async fn new<T>(_args: T) -> Result<Self> {
        let users = Users::new().await?;
        Ok(Memory { users })
    }

    async fn table<'a>(&'a self, name: &str) -> Result<&'a dyn Table> {
        if name == Users::NAME {
            Ok(&self.users)
        } else {
            Err(crate::domain::types::Error::TableNotFound)
        }
    }
}
