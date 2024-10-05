use std::fmt;
use std::error::Error as StdError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DatabaseError {
    ConnectionFailed(String),
}

impl fmt::Display for DatabaseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DatabaseError::ConnectionFailed(msg) => write!(f, "Connection Failed: {}", msg),
        }
    }
}

impl StdError for DatabaseError {}
