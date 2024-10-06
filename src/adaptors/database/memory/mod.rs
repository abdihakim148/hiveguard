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

    async fn table<'a, T: Table + 'a>(&'a self) -> Result<&'a T> {
        if T::NAME == Users::NAME {
            // SAFETY: We know that T is Users, so this cast is safe.
            let users_ref: &Users = &self.users;
            // SAFETY: We know that T is Users, so this cast is safe.
            let table_ref: &T = unsafe { &*(users_ref as *const Users as *const T) };
            Ok(table_ref)
        } else {
            Err(crate::domain::types::Error::Unknown("Table not found".into()))
        }
    }
}
