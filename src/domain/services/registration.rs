/// A trait representing the registration process.
use crate::domain::types::{User, Error};
use crate::ports::{Result, outputs::database::Table};
use bson::oid::ObjectId;

use super::Password;

pub trait Registration: Sized {
    type Id;
    /// Registers a new entity.
    ///
    /// # Returns
    ///
    /// * `Result<Self::Id>` - Returns the ID of the registered entity wrapped in a `Result`.
    async fn register<T: Table<Item = Self, Id = Self::Id>>(&self, table: &T) -> Result<Self>;
}

impl Registration for User {
    type Id = ObjectId;
    async fn register<T: Table<Item = User, Id = ObjectId>>(&self, table: &T) -> Result<Self> {
        let id = self.id;
        let username = self.username.clone();
        let first_name = self.first_name.clone();
        let last_name = self.last_name.clone();
        let email =self.email.clone();
        let password = Password::hash(&self.password)?;
        let mut user = Self{id, username, first_name, last_name, email, password};
        user.id = table.create(&user).await?;
        user.password = Default::default();
        Ok(user)
    }
}
