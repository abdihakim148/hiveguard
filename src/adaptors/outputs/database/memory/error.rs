//! Error types for the memory database implementation
//! 
//! This module defines the error types that can occur during database operations
//! and implements the necessary traits for error handling and HTTP responses.

use crate::domain::types::Error as DomainError;
use std::fmt::{self, Display, Debug};
use std::error::Error as StdError;
#[cfg(feature = "http")]
use actix_web::http::StatusCode;
use std::collections::HashSet;
use crate::ports::ErrorTrait;
use std::sync::PoisonError;
use std::any::type_name;

/// Represents errors that can occur during memory database operations
#[derive(Debug)]
pub enum Error {
    /// User was not found in the database
    UserNotFound,
    /// User already exists with the given email address
    UserWithEmailExists,
    /// User already exists with the given phone number
    UserWithPhoneExists,
    OrganisationNotFound,
    OrganisationWithNameExists,
    MemberNotFound,
    MemberAlreadyExists,
    ServiceNotFound,
    ServiceAlreadyExists,
    CannotDeleteFields(HashSet<String>),
    CannotDeleteContact,
    UnsupportedOperation,
    /// A thread lock was poisoned, indicating a concurrent access failure
    PoisonedLock(&'static str),
    DomainError(DomainError)
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UserNotFound => write!(f, "User not found"),
            Self::UserWithEmailExists => write!(f, "User with this email already exists"),
            Self::UserWithPhoneExists => write!(f, "User with this phone number already exists"),
            Self::OrganisationNotFound => write!(f, "Organisation not found"),
            Self::OrganisationWithNameExists => write!(f, "Organisation with this name already exists"),
            Self::MemberNotFound => write!(f, "Member not found"),
            Self::MemberAlreadyExists => write!(f, "Member already exists in the organisation"),
            Self::ServiceNotFound => write!(f, "service not found"),
            Self::ServiceAlreadyExists => write!(f, "Service with this name already exists"),
            Self::CannotDeleteFields(fields) => {
                if fields.len() == 1 {
                    // unwrap is used here because the above condition makes sure that there is at least one item
                    // which makes this unwrap operation a safe one.
                    write!(f, "cannot delete the {} field", fields.into_iter().next().unwrap())
                } else {
                    write!(f, "cannot delete fields: {}", fields.into_iter().map(|s|s.as_str()).collect::<Vec<&str>>().join(", "))
                }
            },
            Self::CannotDeleteContact => write!(f, "cannot delete the only contact info you have but you can replace it"),
            Self::UnsupportedOperation => write!(f, "Unsupported operation"),
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
            Self::UserWithEmailExists | 
            Self::UserWithPhoneExists | Self::MemberAlreadyExists |
            Self::OrganisationWithNameExists | Self::ServiceAlreadyExists | Self::ServiceAlreadyExists => StatusCode::CONFLICT,
            Self::UserNotFound |
            Self::OrganisationNotFound | Self::ServiceNotFound |
            Self::MemberNotFound | Self::ServiceNotFound=> StatusCode::NOT_FOUND,
            Self::CannotDeleteFields(_) | Self::CannotDeleteContact | Self::UnsupportedOperation => StatusCode::BAD_REQUEST,
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
