/// A trait representing the registration process.
use crate::ports::outputs::database::{CreateItem};
use crate::domain::types::{User, Id};
use crate::ports::outputs::database::Item;
use argon2::PasswordHasher;
use crate::ports::Error;

use super::Password;

pub trait Registration: Sized + Item {
    type Error;
    /// Registers a new entity.
    ///
    /// # Returns
    ///
    /// * returns the created entity.
    async fn register<T: CreateItem<Self>, H: PasswordHasher>(self, db: &T, argon2: &H) -> Result<Self, Self::Error>;
}

impl Registration for User {
    type Error = Error;
    async fn register<T: CreateItem<Self>, H: PasswordHasher>(mut self, db: &T, argon2: &H) -> Result<Self, Self::Error> {
        self.password = self.password.hash(argon2)?;
        let mut user = db.create_item(self).await?;
        user.password = Default::default();
        Ok(user)
    }
}
