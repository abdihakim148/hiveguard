use reqwest::Error as ReqwestError;
use std::error::Error as StdError;
#[cfg(feature = "http")]
use actix_web::http::StatusCode;
use std::fmt::{Display, Debug};
use crate::ports::ErrorTrait;

#[derive(Debug)]
pub enum Error<E = String> {
    InvalidCode,
    Internal(E)
}


impl<E: Display + Debug + Send + Sync + 'static> Error<E> {
    pub fn internal(err: E) -> Self {
        Self::Internal(err)
    }
}


impl From<Error<ReqwestError>> for Error {
    fn from(value: Error<ReqwestError>) -> Self {
        let err = match value {
            Error::InvalidCode => return  Self::InvalidCode,
            Error::Internal(err) => err.to_string()
        };
        Error::internal(err)
    }
}

impl From<Error<serde_json::Error>> for Error {
    fn from(value: Error<serde_json::Error>) -> Self {
        let err = match value {
            Error::InvalidCode => return  Self::InvalidCode,
            Error::Internal(err) => err.to_string()
        };
        Error::internal(err)
    }
}


impl<E: Display + Debug + Send + Sync + 'static> Display for Error<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::InvalidCode => write!(f, "invlid code"),
            Error::Internal(err) => Display::fmt(err, f)
        }
    }
}


impl<E: Display + Debug + Send + Sync + 'static> StdError for Error<E> {}


impl<E: Display + Debug + Send + Sync + 'static> ErrorTrait for Error<E> {
    fn log_message(&self) -> String {
        match self {
            Error::InvalidCode => format!("invlid code"),
            Error::Internal(err) => format!("internal error: {}", err)
        }
    }

    fn user_message(&self) -> String {
        String::from("internal server error occured")
    }

    #[cfg(feature = "http")]
    fn status(&self) -> StatusCode {
        match self {
            Error::InvalidCode => StatusCode::BAD_REQUEST,
            Error::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}