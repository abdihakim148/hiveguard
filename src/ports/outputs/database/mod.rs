/// Module for database table operations.
mod table;
mod item;

use crate::domain::types::{ErrorTrait, Error};
use crate::domain::types::User;
pub use table::Table;
pub use item::Item;

/// A trait representing a database with user-related operations.
pub trait Database: Sized {
    type Users: Table;
    type Config;
    type Error: ErrorTrait + Into<Error>;
    async fn new(config: Self::Config) -> Result<Self, Self::Error>;
    async fn users<'a>(&'a self) -> Result<&'a Self::Users, Self::Error>;
}
