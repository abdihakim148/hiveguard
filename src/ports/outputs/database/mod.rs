#![allow(unused)]
mod table;

pub use crate::domain::types::{Result, Error};
pub use table::Table;

/// A trait representing a database with user-related operations.
pub trait Database: Sized {
    type Users: Table;
    async fn new<T>(config: T) -> Result<Self>;
    async fn users<'a>(&'a self) -> &'a Self::Users;
}
