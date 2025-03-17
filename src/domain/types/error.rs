use argon2::password_hash::Error as HashError;
use rusty_paseto::core::PasetoError;
use std::fmt::{self, Display, Debug};
use std::error::Error as StdError;
use lettre::address::AddressError;
#[cfg(feature = "http")]
use actix_web::http::StatusCode;
use crate::ports::ErrorTrait;
use serde::Serialize;

#[derive(Debug)]
pub enum Error {
    // Authentication errors
    WrongPassword,
    InvalidEmail,
    InvalidPhone,
    TokenExpired,
    InvalidToken,
    EmailAddressRequired,
    PhoneNumberRequired,
    
    // Resource errors
    ResourceNotFound { resource: String },
    DuplicateResource { resource: String },
    
    // Validation errors
    ValidationError { 
        field: String, 
        message: String 
    },
    
    // Format errors
    InvalidFormat { 
        expected: String,
        found: String,
        field: Option<String>
    },
    
    // Internal errors (not serialized to user)
    Internal { 
        message: String,
        source: Option<Box<dyn StdError + Send + Sync>>
    },
    New(Box<dyn ErrorTrait + Send + Sync>)
}

impl Error {
    pub fn new<T: ErrorTrait + Send + Sync>(err: T) -> Self {
        Error::New(Box::new(err))
    }
}

// Email-related error conversions
impl From<AddressError> for Error {
    fn from(_: AddressError) -> Self {
        Error::InvalidEmail
    }
}

// Password-related error conversions
impl From<HashError> for Error {
    fn from(err: HashError) -> Self {
        match err {
            HashError::Password => Error::WrongPassword,
            _ => Error::internal(err)
        }
    }
}

// Token-related error conversions
impl From<PasetoError> for Error {
    fn from(err: PasetoError) -> Self {
        match err {
            PasetoError::InvalidSignature => Error::InvalidToken,
            // PasetoError::FooterInvalid | PasetoError::WrongHeader,
            _ => Error::internal(err),
        }
    }
}

// Standard error conversions
impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::internal(err)
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Error::internal(err)
    }
}

impl Error {
    pub fn internal<E>(error: E) -> Self 
    where 
        E: StdError + Send + Sync + 'static 
    {
        Self::Internal { 
            message: "an internal error occurred".to_string(),
            source: Some(Box::new(error))
        }
    }

    pub fn validation(field: impl Into<String>, message: impl Into<String>) -> Self {
        Self::ValidationError {
            field: field.into(),
            message: message.into()
        }
    }

    pub fn invalid_format(expected: impl Into<String>, found: impl Into<String>, field: Option<String>) -> Self {
        Self::InvalidFormat {
            expected: expected.into(),
            found: found.into(),
            field
        }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::WrongPassword => write!(f, "Invalid password"),
            Self::InvalidEmail => write!(f, "Invalid email format"),
            Self::InvalidPhone => write!(f, "Invalid phone number format"),
            Self::TokenExpired => write!(f, "Token has expired"),
            Self::InvalidToken => write!(f, "Invalid token"),
            Self::EmailAddressRequired => write!(f, "email address is required"),
            Self::PhoneNumberRequired => write!(f, "phone number is required"),
            Self::ResourceNotFound { resource } => write!(f, "{} not found", resource),
            Self::DuplicateResource { resource } => write!(f, "{} already exists", resource),
            Self::ValidationError { field, message } => write!(f, "{}: {}", field, message),
            Self::InvalidFormat { expected, found, field } => {
                if let Some(field) = field {
                    write!(f, "Expected {} but found {} for field {}", expected, found, field)
                } else {
                    write!(f, "Expected {} but found {}", expected, found)
                }
            },
            Self::Internal { message, .. } => write!(f, "{}", message),
            Self::New(err) => Display::fmt(err, f)
        }
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Self::Internal { source, .. } => source.as_ref().map(|e| e.as_ref() as &(dyn StdError + 'static)),
            _ => None
        }
    }
}

impl ErrorTrait for Error {
    fn log_message(&self) -> String {
        match self {
            Self::Internal { message, source } => {
                if let Some(err) = source {
                    format!("Internal error: {}", err)
                } else {
                    format!("Internal error: {}", message)
                }
            },
            Self::New(err) => err.log_message(),
            _ => self.to_string()
        }
    }

    fn user_message(&self) -> String {
        match self {
            Self::Internal { .. } => "An internal error occurred".to_string(),
            Self::New(err) => err.user_message(),
            _ => self.to_string()
        }
    }

    #[cfg(feature = "http")]
    fn status(&self) -> StatusCode {
        match self {
            Self::WrongPassword |
            Self::TokenExpired | 
            Self::InvalidToken => StatusCode::UNAUTHORIZED,
            Self::InvalidEmail |
            Self::InvalidPhone |
            Self::ValidationError { .. } |
            Self::InvalidFormat { .. } | Self::PhoneNumberRequired | Self::EmailAddressRequired => StatusCode::BAD_REQUEST,
            Self::ResourceNotFound { .. } => StatusCode::NOT_FOUND,
            Self::DuplicateResource { .. } => StatusCode::CONFLICT,
            Self::Internal { .. } => StatusCode::INTERNAL_SERVER_ERROR,
            Self::New(err) => err.status()
        }
    }
}


