#![allow(unused)]
use std::fmt;
use std::error::Error as StdError;
mod database_error;

pub use database_error::DatabaseError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    NotFound,
    LockError(String),
    EmailAlreadyExists,
    UserNotFound,
    InvalidUserId,
    DatabaseConsistencyError,
    SerializationError(String),
    InvalidInput(String),
    Database(DatabaseError),
    Unauthorized,
    Unknown(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::NotFound => write!(f, "Not Found"),
            Error::InvalidInput(msg) => write!(f, "Invalid Input: {}", msg),
            Error::Database(err) => write!(f, "Database Error: {}", err),
            Error::Unauthorized => write!(f, "Unauthorized"),
            Error::Unknown(msg) => write!(f, "Unknown Error: {}", msg),
            Error::LockError(msg) => write!(f, "Lock Error: {}", msg),
            Error::EmailAlreadyExists => write!(f, "Email Already Exists"),
            Error::UserNotFound => write!(f, "User Not Found"),
            Error::InvalidUserId => write!(f, "Invalid User ID"),
            Error::DatabaseConsistencyError => write!(f, "Database Consistency Error"),
            Error::SerializationError(msg) => write!(f, "Serialization Error: {}", msg),
        }
    }
}

impl StdError for Error {}

impl From<DatabaseError> for Error {
    fn from(error: DatabaseError) -> Self {
        Error::Database(error)
    }
}
