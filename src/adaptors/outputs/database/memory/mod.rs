/// Module for database tables.
mod tables;

use crate::ports::outputs::database::{Database, Result};
use crate::ports::outputs::database::Table;
use tokio::sync::OnceCell;
pub use tables::*;


pub static MEMORY: OnceCell<Memory> = OnceCell::const_new();

/// A struct representing an in-memory database.
/// A struct representing an in-memory database.
pub struct Memory {
    users: Users,
}

impl Database for Memory {
    type Users = Users;
    type Config = ();

    async fn new(_config: ()) -> Result<Self> {
        let users = Users::new().await?;
        Ok(Memory { users })
    }

    async fn users<'a>(&'a self) -> Result<&'a Self::Users> {
        Ok(&self.users)
    }
}
