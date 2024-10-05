pub mod table;

pub use crate::domain::types::error::DatabaseError;

type Result<T> = std::result::Result<T, DatabaseError>;

pub trait Database: Sized {
    async fn new<T>(args: T) -> Result<Self>;
}
