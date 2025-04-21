use lettre::transport::smtp::Error as SmtpError;
use lettre::error::Error as LettreError;
use reqwest::Error as ReqwestError;
use std::error::Error as StdError;
#[cfg(feature = "http")]
use actix_web::http::StatusCode;
use std::fmt::{Display, Debug};
use crate::ports::ErrorTrait;

#[derive(Debug)]
pub enum Error {
    InvalidCode,
    Internal(Box<dyn StdError + Send + Sync + 'static>),
    Err(Box<dyn ErrorTrait + 'static>)
}


impl Error {
    pub fn internal(err: impl Into<Box<dyn StdError + Send + Sync + 'static>>) -> Self {
        Self::Internal(err.into())
    }

    pub fn err<T: ErrorTrait + 'static>(err: T) -> Self {
        Self::Err(Box::new(err))
    }
}


impl From<SmtpError> for Error {
    fn from(err: SmtpError) -> Self {
        Error::Internal(Box::new(err))
    }
}


impl From<LettreError> for Error {
    fn from(err: LettreError) -> Self {
        match err {
            LettreError::Io(err) => Error::internal(err),
            _ => Error::Err(Box::new(crate::domain::types::Error::InvalidEmail))
        }
    }
}


impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::InvalidCode => write!(f, "invlid code"),
            Error::Internal(err) => Display::fmt(err, f),
            Error::Err(err) => Display::fmt(err, f)
        }
    }
}


impl StdError for Error {}


impl ErrorTrait for Error {
    fn log_message(&self) -> String {
        match self {
            Error::InvalidCode => String::from("invlid code"),
            Error::Internal(err) => format!("internal error: {}", err),
            Error::Err(err) => err.log_message()
        }
    }

    fn user_message(&self) -> String {
        match self {
            Error::InvalidCode => String::from("invlid code"),
            Error::Internal(_) => String::from("internal server error occured"),
            Error::Err(err) => err.user_message()
        }
    }

    #[cfg(feature = "http")]
    fn status(&self) -> StatusCode {
        match self {
            Error::InvalidCode => StatusCode::BAD_REQUEST,
            Error::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Error::Err(err) => err.status()
        }
    }
}
