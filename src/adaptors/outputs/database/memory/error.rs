use crate::ports::Error as GlobalError;
use std::sync::PoisonError;
use thiserror::Error as ThisError;
#[cfg(feature = "actix")]
use actix_web::{ResponseError, http::StatusCode, body::BoxBody, HttpResponse};


#[derive(ThisError, Debug)]
pub enum Error {
    #[error("Lock Poisoned")]
    LockPoisoned,
    #[error("user with the same email already exists")]
    UserWithEmailExists,
    #[error("{0} not found")]
    NotFound(&'static str),
    #[error("data inconsistency")]
    InconsistentData,
}


#[cfg(feature = "actix")]
impl ResponseError for Error {
    fn status_code(&self) -> StatusCode {
        match self {
            Self::LockPoisoned | Self::InconsistentData => StatusCode::INTERNAL_SERVER_ERROR,
            Self::UserWithEmailExists => StatusCode::CONFLICT,
            Self::NotFound(_) => StatusCode::NOT_FOUND,
        }
    }

    fn error_response(&self) -> HttpResponse<BoxBody> {
        let status = self.status_code();
        match self {
            Self::LockPoisoned | Self::InconsistentData => HttpResponse::with_body(status, BoxBody::new(format!("{{\"error\": \"Internal Server Error\"}}",))),
            Self::NotFound(_) | Self::UserWithEmailExists => HttpResponse::with_body(status, BoxBody::new(format!("{{\"error\": \"{self}\"}}")))
        }
    }
}


impl From<Error> for GlobalError {
    fn from(err: Error) -> Self {
        GlobalError::new(err)
    }
}

impl<T> From<PoisonError<T>> for Error {
    fn from(_: PoisonError<T>) -> Self {
        Self::LockPoisoned
    }
}