use std::fmt::{Display, Formatter};
use std::error::Error as StdError;
use actix_web::http::StatusCode;
use crate::ports::ErrorTrait;

#[derive(Debug)]
pub enum Error {
    UnAuthorized
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnAuthorized => write!(f, "unauthorized request")
        }
    }
}

impl StdError for Error{}

impl ErrorTrait for Error {
    fn log_message(&self) -> String {
        self.to_string()
    }
    // test comment

    #[cfg(feature = "http")]
    fn status(&self) -> StatusCode {
        StatusCode::UNAUTHORIZED
    }

    fn user_message(&self) -> String {
        self.to_string()
    }
}