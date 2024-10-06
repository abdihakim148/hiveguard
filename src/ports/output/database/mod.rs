#![allow(unused)]
mod table;

pub use crate::domain::types::Error;
pub use table::Table;

pub type Result<T> = std::result::Result<T, Error>;

pub trait Database: Sized {
    async fn new<T>(args: T) -> Result<Self>;
}
