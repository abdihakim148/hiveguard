mod database_error;

pub use database_error::DatabaseError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    NotFound,
    InvalidInput(String),
    Database(DatabaseError),
    Unauthorized,
    Unknown(String),
}

impl From<DatabaseError> for Error {
    fn from(error: DatabaseError) -> Self {
        Error::Database(error)
    }
}
