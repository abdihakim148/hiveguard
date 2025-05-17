use std::fmt::{Display, Formatter};
use std::error::Error as StdError;

#[derive(Debug, Clone, PartialEq)]
pub enum Error {
    InvalidEmailAddress,
    InvalidPhoneNumber,
    UnsupportedOAuthProvider(String),
    InvalidId(String)
}


impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::InvalidEmailAddress => write!(f, "invalid email address"),
            Error::InvalidPhoneNumber => write!(f, "invalid phone number"),
            Error::UnsupportedOAuthProvider(provider) => write!(f, "unsupported OAuth provider: {}", provider),
            Error::InvalidId(id) => write!(f, "invalid id: {}", id),
        }
    }
}


impl StdError for Error{}