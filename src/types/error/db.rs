#[cfg(feature = "dynamodb")]
use aws_sdk_dynamodb::error::{SdkError, BuildError};
use std::fmt::{Display, Formatter};
use std::error::Error as StdError;
use super::ConversionError;


#[derive(Debug)]
pub enum DatabaseError {
    UserNotFound,
    SessionNotFound,
    VerificationNotFound,
    ConversionError(ConversionError),
    Internal(Box<dyn StdError + Send + Sync>)
}


impl Display for DatabaseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            DatabaseError::UserNotFound => write!(f, "user not found"),
            DatabaseError::SessionNotFound => write!(f, "session not found"),
            DatabaseError::VerificationNotFound => write!(f, "verification not found"),
            DatabaseError::ConversionError(err) => write!(f, "conversion error: {}", err),
            DatabaseError::Internal(err) => write!(f, "internal error: {}", err)
        }
    }
}


impl StdError for DatabaseError {}


impl PartialEq for DatabaseError {
    fn eq(&self, other: &Self) -> bool {
        match self {
            DatabaseError::UserNotFound => match other {DatabaseError::UserNotFound => true, _ => false},
            DatabaseError::SessionNotFound => match other {DatabaseError::SessionNotFound => true, _ => false},
            DatabaseError::VerificationNotFound => match other {DatabaseError::VerificationNotFound => true, _ => false},
            DatabaseError::ConversionError(err) => match other {DatabaseError::ConversionError(other_err) => err == other_err, _ => false},
            DatabaseError::Internal(err) => match other {DatabaseError::Internal(other_err) => err.to_string() == other_err.to_string(), _ => false},
        }
    }
}

#[cfg(feature = "dynamodb")]
impl<E: StdError + 'static + Send + Sync, R: std::fmt::Debug + Send + Sync + 'static> From<SdkError<E, R>> for DatabaseError {
    fn from(err: SdkError<E, R>) -> Self {
        DatabaseError::Internal(Box::new(err))
    }
}

#[cfg(feature = "dynamodb")]
impl From<BuildError> for DatabaseError {
    fn from(err: BuildError) -> Self {
        DatabaseError::Internal(Box::new(err))
    }
}


impl From<ConversionError> for DatabaseError {
    fn from(err: ConversionError) -> Self {
        DatabaseError::ConversionError(err)
    }
}