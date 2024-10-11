/// A trait representing the registration process.
use crate::domain::types::{User, Error, Result};
use bson::oid::ObjectId;

use super::Password;

pub trait Registration: Sized {
    /// Registers a new entity.
    ///
    /// # Returns
    ///
    /// * `Result<Self::Id>` - Returns the ID of the registered entity wrapped in a `Result`.
    fn register(&self) -> Result<Self>;
}

impl Registration for User {
    fn register(&self) -> Result<Self> {
        let id = self.id;
        let username = self.username.clone();
        let first_name = self.first_name.clone();
        let last_name = self.last_name.clone();
        let email =self.email.clone();
        let password = Password::hash(&self.password)?;
        Ok(Self{id, username, first_name, last_name, email, password})
    }
}
