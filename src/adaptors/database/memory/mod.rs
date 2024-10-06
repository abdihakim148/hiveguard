mod tables;

use crate::ports::output::database::{Database, Result};
use tables::users::Users;

/// A struct representing an in-memory database.
pub struct Memory {
    users: Users,
}

#[async_trait::async_trait]
impl Database for Memory {
    async fn new<T>(_args: T) -> Result<Self> {
        let users = Users::new().await?;
        Ok(Memory { users })
    }

    async fn table<'a, T: Table + 'a>(&'a self) -> Result<&'a T> {
        // This is a placeholder implementation. You will need to adjust this to return the correct table.
        Err(crate::domain::types::Error::Unknown("Table not found".into()))
    }
}
