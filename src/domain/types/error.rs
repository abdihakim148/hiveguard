use argon2::password_hash::Error as HashError;
use rusty_paseto::core::PasetoError;
use std::fmt::{self, Display, Debug};
use std::error::Error as StdError;
use std::sync::Arc;
use std::string::FromUtf8Error;
use base64::DecodeError;
use lettre::address::AddressError;
use jsonwebtoken::errors::{Error as JwtLibError, ErrorKind as JwtLibErrorKind};
// Removed unnecessary ring import
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
    ContactAlreadyVerified,
    /// this is supposed to be returned when a contact is:
    /// only email and the current feature is phone or
    /// only phone and the current feature is phone.
    ContactFeatureConflict,
    UnknownAlgorithm, // Added for unknown JWT algorithm
    JwtError(JwtLibError), // Added for JWT library errors

    // Authentication method errors
    IncorrectLoginMethod,
    IncorrectSocialProvider { 
        expected: String, 
        found: String 
    },
    SocialProviderNotFound { 
        provider: String 
    },
    CouldNotGetEmail(Box<dyn StdError + Send + Sync + 'static>),
    CouldNotGetPhone(Box<dyn StdError + Send + Sync + 'static>),
    CouldNotGetNecessaryInfo(Box<dyn StdError + Send + Sync + 'static>),
    
    /// Returned when an invalid authorization code is provided
    IncorrectCode,
    
    // Resource errors
    ItemNotFound(&'static str),
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

// JWT error conversion
impl From<JwtLibError> for Error {
    fn from(err: JwtLibError) -> Self {
        Error::JwtError(err)
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
        E: Into<Box<dyn StdError + Send + Sync + 'static>> 
    {
        Self::Internal { 
            message: "an internal error occurred".to_string(),
            source: Some(error.into())
        }
    }
    
    pub fn could_not_get_email<E>(error: E) -> Self 
    where 
        E: Into<Box<dyn StdError + Send + Sync + 'static>> 
    {
        Self::CouldNotGetEmail(error.into())
    }

    pub fn could_not_get_phone<E>(error: E) -> Self 
    where 
        E: Into<Box<dyn StdError + Send + Sync + 'static>> 
    {
        Self::CouldNotGetPhone(error.into())
    }

    pub fn could_not_get_necessary_info<E>(error: E) -> Self 
    where 
        E: Into<Box<dyn StdError + Send + Sync + 'static>> 
    {
        Self::CouldNotGetNecessaryInfo(error.into())
    }

    pub fn item_not_found(item_name: &'static str) -> Self {
        Self::ItemNotFound(item_name)
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
            Self::ContactFeatureConflict => write!(f, "current feature and a user contact do not align"),
            Self::EmailAddressRequired => write!(f, "email address is required"),
            Self::PhoneNumberRequired => write!(f, "phone number is required"),
            Self::ContactAlreadyVerified => write!(f, "this contact is already verified"),
            Self::UnknownAlgorithm => write!(f, "Unknown or unsupported algorithm specified"), // Added arm
            Self::JwtError(err) => write!(f, "JWT Error: {}", err), // Added arm
            Self::IncorrectLoginMethod =>
                write!(f, "Incorrect login method"),
            Self::IncorrectSocialProvider { expected, found } =>
                write!(f, "Incorrect social provider. Expected {}, found {}", expected, found),
            Self::SocialProviderNotFound { provider } =>
                write!(f, "Social provider {} not found", provider),
            Self::CouldNotGetEmail(_) =>
                write!(f, "Failed to retrieve email from the provider. Please ensure the account has a verified email accessible to us, or try another login method."),
            Self::CouldNotGetPhone(_) =>
                write!(f, "Failed to retrieve phone number from the provider. Please ensure the account has a verified phone number accessible to us, or try another login method."),
            Self::CouldNotGetNecessaryInfo(_) =>
                write!(f, "Failed to retrieve necessary information from the provider. Please ensure the account profile is complete and accessible, or try another login method."),
            Self::IncorrectCode =>
                write!(f, "Invalid authorization code"),
            Self::ItemNotFound(item_name) => write!(f, "{} not found", item_name),
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
            Self::CouldNotGetEmail(err) | Self::CouldNotGetPhone(err) | Self::CouldNotGetNecessaryInfo(err) => Some(err.as_ref()),
            Self::JwtError(err) => Some(err as &(dyn StdError + 'static)), // Added arm
            _ => None // UnknownAlgorithm has no source
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
            Self::CouldNotGetEmail(err) => format!("Could not get email from provider: {}", err),
            Self::CouldNotGetPhone(err) => format!("Could not get phone from provider: {}", err),
            Self::CouldNotGetNecessaryInfo(err) => format!("Could not get necessary info from provider: {}", err),
            Self::New(err) => err.log_message(),
            Self::JwtError(err) => format!("JWT processing error: {}", err), // Added arm
            Self::UnknownAlgorithm => "JWT verification failed due to unknown algorithm".to_string(), // Added arm
            _ => self.to_string() // Other errors use Display impl
        }
    }

    fn user_message(&self) -> String {
        match self {
            // Cases returning generic internal error message
            Self::Internal { .. } | Self::ContactFeatureConflict | Self::UnknownAlgorithm => "An internal error occurred".to_string(), // Added UnknownAlgorithm
            // Cases returning specific, user-friendly messages from Display impl (which now includes suggestion)
            Self::CouldNotGetEmail(_) | Self::CouldNotGetPhone(_) | Self::CouldNotGetNecessaryInfo(_) => self.to_string(),
            // Handle JWT errors based on kind
            Self::JwtError(err) => match err.kind() { // Added arm for JwtError
                JwtLibErrorKind::InvalidToken | JwtLibErrorKind::InvalidSignature | JwtLibErrorKind::MissingAlgorithm => "Invalid or malformed token provided.".to_string(),
                JwtLibErrorKind::ExpiredSignature => "The provided token has expired.".to_string(),
                JwtLibErrorKind::InvalidIssuer => "The token issuer is invalid.".to_string(),
                JwtLibErrorKind::InvalidAudience => "The token audience is invalid.".to_string(),
                JwtLibErrorKind::InvalidSubject => "The token subject is invalid.".to_string(),
                JwtLibErrorKind::ImmatureSignature => "The token is not yet valid.".to_string(),
                JwtLibErrorKind::InvalidAlgorithm => "The token algorithm is invalid or not supported.".to_string(),
                JwtLibErrorKind::MissingRequiredClaim(claim) => format!("Missing required claim: {}", claim),
                // Internal/unexpected JWT errors
                _ => "An internal error occurred while processing the token.".to_string(),
            },
            // Delegate to wrapped error
            Self::New(err) => err.user_message(),
            // Default: use Display impl
            _ => self.to_string()
        }
    }

    #[cfg(feature = "http")]
    fn status(&self) -> StatusCode {
        match self {
            Self::WrongPassword |
            Self::TokenExpired |
            Self::InvalidToken |
            Self::IncorrectLoginMethod |
            Self::IncorrectSocialProvider { .. } |
            Self::IncorrectCode => StatusCode::UNAUTHORIZED,
            Self::JwtError(err) => match err.kind() { // Added arm for JwtError
                JwtLibErrorKind::InvalidToken |
                JwtLibErrorKind::InvalidSignature |
                JwtLibErrorKind::ExpiredSignature |
                JwtLibErrorKind::InvalidIssuer |
                JwtLibErrorKind::InvalidAudience |
                JwtLibErrorKind::InvalidSubject |
                JwtLibErrorKind::ImmatureSignature |
                JwtLibErrorKind::InvalidAlgorithm |
                JwtLibErrorKind::MissingAlgorithm => StatusCode::UNAUTHORIZED,
                JwtLibErrorKind::MissingRequiredClaim(_) => StatusCode::BAD_REQUEST,
                _ => StatusCode::INTERNAL_SERVER_ERROR, // Treat other JWT errors as internal
            },
            Self::SocialProviderNotFound { .. } => StatusCode::NOT_FOUND,
            Self::InvalidEmail |
            Self::InvalidPhone |
            Self::ValidationError { .. } |
            Self::InvalidFormat { .. } | Self::PhoneNumberRequired | Self::EmailAddressRequired | Self::ContactAlreadyVerified => StatusCode::BAD_REQUEST,
            Self::ItemNotFound(_) | Self::ResourceNotFound { .. } => StatusCode::NOT_FOUND,
            Self::DuplicateResource { .. } => StatusCode::CONFLICT,
            Self::Internal { .. } | Self::ContactFeatureConflict | Self::UnknownAlgorithm => StatusCode::INTERNAL_SERVER_ERROR, // Added UnknownAlgorithm
            Self::CouldNotGetEmail(_) | Self::CouldNotGetPhone(_) | Self::CouldNotGetNecessaryInfo(_) => StatusCode::BAD_GATEWAY,
            Self::New(err) => err.status()
        }
    }
}


