/// A trait representing the registration process.
use crate::domain::types::{User, Error};
use bson::oid::ObjectId;

pub trait Registration {
    /// The type of the identifier for the registered entity.
    type Id;

    /// Registers a new entity.
    ///
    /// # Returns
    ///
    /// * `Result<Self::Id>` - Returns the ID of the registered entity wrapped in a `Result`.
    fn register(&self) -> Result<Self::Id, crate::domain::types::Error>;
}

impl Registration for User {
    type Id = ObjectId;

    fn register(&self) -> Result<Self::Id, Error> {
        // Here you would typically add logic to register the user, such as saving to a database.
        // For now, we'll just return the user's ID.
        Ok(self.id.clone())
    }
}
