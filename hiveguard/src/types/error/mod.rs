pub use conversion::ConversionError;
use std::fmt::{Display, Formatter};
use std::error::Error as StdError;
pub use db::DatabaseError;

mod db;
mod conversion;

#[derive(Debug, PartialEq)]
pub enum Error {
    ConversionError(ConversionError),
    DatabaseError(DatabaseError)
}


impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::ConversionError(err) => write!(f, "conversion error: {}", err),
            Error::DatabaseError(err) => write!(f, "database error: {}", err)
        }
    }
}


impl StdError for Error{}


impl From<DatabaseError> for Error {
    fn from(err: DatabaseError) -> Self {
        Error::DatabaseError(err)
    }
}

impl From<ConversionError> for Error {
    fn from(err: ConversionError) -> Self {
        Error::ConversionError(err)
    }
}