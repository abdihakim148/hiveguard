#![allow(unused)]
mod table;

pub use crate::domain::types::Error;
pub use table::Table;

pub type Result<T> = std::result::Result<T, Error>;

/// A trait representing a database with user-related operations.
pub trait Database: Sized {
    type Users: Table;
    async fn new<T>(config: T) -> Result<Self>;
    async fn users<'a>(&'a self) -> &'a Self::Users;
}
