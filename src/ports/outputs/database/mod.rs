/// Module for database table operations.
mod table;
mod item;

use crate::domain::types::{Error, User, Organisation, Resource, Role, Member, Verification, Service, Scope};
pub use table::Table;
pub use item::Item;

/// A trait representing a database with user-related operations.
/// A trait representing a database with user-related operations.
/// 
/// This trait defines the structure and operations for a database that manages
/// various entities such as verifications, organisations, resources, services,
/// members, scopes, users, and roles. It provides asynchronous methods to access
/// these entities and perform operations on them.
/// A trait representing a database with user-related operations.
///
/// This trait defines the structure and operations for a database that manages
/// various entities such as verifications, organisations, resources, services,
/// members, scopes, users, and roles. It provides asynchronous methods to access
/// these entities and perform operations on them.
pub trait Database: Sized {
    /// The table type for verifications.
    /// The table type for verifications.
    type Verifications: Table<Item = Verification>;
    /// The table type for organisations.
    type Organisations: Table<Item = Organisation>;
    /// The table type for resources.
    type Resources: Table<Item = Resource>;
    /// The table type for services.
    type Services: Table<Item = Service>;
    /// The table type for members.
    type Members: Table<Item = Member>;
    /// The table type for scopes.
    type Scopes: Table<Item = Scope>;
    /// The table type for users.
    type Users: Table<Item = User>;
    /// The table type for roles.
    type Roles: Table<Item = Role>;
    /// The configuration type for the database.
    type Config;
    /// The error type for database operations.
    type Error: Into<Error>;

    /// Creates a new instance of the database with the given configuration.
    ///
    /// # Arguments
    ///
    /// * `config` - The configuration for the database.
    ///
    /// # Returns
    ///
    /// * `Result<Self, Self::Error>` - Returns an instance of the database or an error.

    async fn new(config: Self::Config) -> Result<Self, Self::Error>;
    /// Retrieves the verifications table.
    ///
    /// # Returns
    ///
    /// * `Result<&'a Self::Verifications, Self::Error>` - Returns a reference to the verifications table or an error.
    async fn verifications<'a>(&'a self) -> Result<&'a Self::Verifications, Self::Error>;
    /// Retrieves the organisations table.
    ///
    /// # Returns
    ///
    /// * `Result<&'a Self::Organisations, Self::Error>` - Returns a reference to the organisations table or an error.
    async fn organisations<'a>(&'a self) -> Result<&'a Self::Organisations, Self::Error>;
    /// Retrieves the resources table.
    ///
    /// # Returns
    ///
    /// * `Result<&'a Self::Resources, Self::Error>` - Returns a reference to the resources table or an error.
    async fn resources<'a>(&'a self) -> Result<&'a Self::Resources, Self::Error>;
    /// Retrieves the services table.
    ///
    /// # Returns
    ///
    /// * `Result<&'a Self::Services, Self::Error>` - Returns a reference to the services table or an error.
    async fn services<'a>(&'a self) -> Result<&'a Self::Services, Self::Error>;
    /// Retrieves the members table.
    ///
    /// # Returns
    ///
    /// * `Result<&'a Self::Members, Self::Error>` - Returns a reference to the members table or an error.
    async fn members<'a>(&'a self) -> Result<&'a Self::Members, Self::Error>;
    /// Retrieves the scopes table.
    ///
    /// # Returns
    ///
    /// * `Result<&'a Self::Scopes, Self::Error>` - Returns a reference to the scopes table or an error.
    async fn scopes<'a>(&'a self) -> Result<&'a Self::Scopes, Self::Error>;
    /// Retrieves the users table.
    ///
    /// # Returns
    ///
    /// * `Result<&'a Self::Users, Self::Error>` - Returns a reference to the users table or an error.
    async fn users<'a>(&'a self) -> Result<&'a Self::Users, Self::Error>;
    /// Retrieves the roles table.
    ///
    /// # Returns
    ///
    /// * `Result<&'a Self::Roles, Self::Error>` - Returns a reference to the roles table or an error.
    async fn roles<'a>(&'a self) -> Result<&'a Self::Roles, Self::Error>;
}
