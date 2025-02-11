//! Error types for the memory database implementation
//! 
//! This module defines the error types that can occur during database operations
//! and implements the necessary traits for error handling and HTTP responses.

use crate::domain::types::Error as DomainError;
use std::fmt::{self, Display, Debug};
use std::error::Error as StdError;
#[cfg(feature = "http")]
use actix_web::http::StatusCode;
use crate::ports::ErrorTrait;
use std::sync::PoisonError;
use std::any::type_name;

/// Represents errors that can occur during memory database operations
#[derive(Debug)]
pub enum Error {
    /// User was not found in the database
    /// User already exists with the given email address
    /// User already exists with the given phone number
    /// A thread lock was poisoned, indicating a concurrent access failure
    UserNotFound,
    UserWithEmailExists,
    UserWithPhoneExists,
    PoisonedLock(&'static str),
    DomainError(DomainError)
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UserNotFound => write!(f, "User not found"),
            Self::UserWithEmailExists => write!(f, "User with this email already exists"),
            Self::UserWithPhoneExists => write!(f, "User with this phone number already exists"),
            Self::PoisonedLock(lock_type) => write!(f, "Thread lock poisoned for {}", lock_type),
            Self::DomainError(err) => Display::fmt(err, f)
        }
    }
}

impl StdError for Error {}

impl ErrorTrait for Error {
    fn log_message(&self) -> String {
        match self {
            Self::PoisonedLock(lock_type) => 
                format!("Critical: Thread lock poisoned for type {}", lock_type),
            _ => self.to_string()
        }
    }

    fn user_message(&self) -> String {
        match self {
            Self::PoisonedLock(_) => 
                "An internal error occurred".to_string(),
            _ => self.to_string()
        }
    }

    #[cfg(feature = "http")]
    fn status(&self) -> StatusCode {
        match self {
            Self::UserNotFound => StatusCode::NOT_FOUND,
            Self::UserWithEmailExists | 
            Self::UserWithPhoneExists => StatusCode::CONFLICT,
            Self::PoisonedLock(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::DomainError(err) => err.status()
        }
    }
}

impl<T> From<PoisonError<T>> for Error {
    fn from(_: PoisonError<T>) -> Self {
        Self::PoisonedLock(type_name::<T>())
    }
}


impl From<DomainError> for Error {
    fn from(err: DomainError) -> Self {
        Self::DomainError(err)
    }
}