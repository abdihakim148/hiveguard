mod table;

pub use crate::domain::types::error::DatabaseError;
pub use table::Table;

type Result<T> = std::result::Result<T, DatabaseError>;

pub trait Database: Sized {
    async fn new<T>(args: T) -> Result<Self>;
    async fn table<'a, T: Table + 'a>(&'a self) -> Result<&'a T>;
}
