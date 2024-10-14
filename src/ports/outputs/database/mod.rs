#![allow(unused)]
/// Module for database table operations.
mod table;

pub use crate::domain::types::{Result, Error};
pub use table::Table;

/// A trait representing a database with user-related operations.
pub trait Database: Sized {
    type Users: Table;
    type Config;
    async fn new(config: Self::Config) -> Result<Self>;
    async fn users<'a>(&'a self) -> &'a Self::Users;
}
