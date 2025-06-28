pub use password_hash::errors::Error as HashError;
pub use conversion::ConversionError;
use std::fmt::{Display, Formatter};
use std::error::Error as StdError;
pub use db::DatabaseError;

mod db;
mod conversion;

#[derive(Debug, PartialEq)]
pub enum Error {
    ConversionError(ConversionError),
    DatabaseError(DatabaseError),
    HashError(HashError),
    WrongPassword,
}


impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::ConversionError(err) => write!(f, "conversion error: {}", err),
            Error::DatabaseError(err) => write!(f, "database error: {}", err),
            Error::HashError(err) => write!(f, "hash error: {}", err),
            Error::WrongPassword => write!(f, "wrong password"),
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

impl From<HashError> for Error {
    fn from(err: HashError) -> Self {
        match err {
            HashError::Password => Error::WrongPassword,
            _ => Error::HashError(err)
        }
    }
}