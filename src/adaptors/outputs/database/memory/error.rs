use crate::ports::{ErrorTrait, Error as GlobalError};
use thiserror::Error as ThisError;
use crate::domain::Error as DomainError;
use std::sync::PoisonError;
#[cfg(feature = "actix")]
use actix_web::{ResponseError, http::StatusCode, body::BoxBody, HttpResponse, HttpResponseBuilder as ResponseBuilder};


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
    #[error("{0}")]
    New(Box<dyn ErrorTrait>)
}


#[cfg(feature = "actix")]
impl ResponseError for Error {
    fn status_code(&self) -> StatusCode {
        match self {
            Self::LockPoisoned | Self::InconsistentData => StatusCode::INTERNAL_SERVER_ERROR,
            Self::UserWithEmailExists => StatusCode::CONFLICT,
            Self::NotFound(_) => StatusCode::NOT_FOUND,
            Self::New(err) => err.status_code(),
        }
    }

    fn error_response(&self) -> HttpResponse<BoxBody> {
        let status = self.status_code();
        let mut builder = ResponseBuilder::new(status);
        builder.content_type("application/json");
        let body = match self {
            Self::LockPoisoned | Self::InconsistentData => BoxBody::new(format!("{{\"error\": \"Internal Server Error\"}}",)),
            Self::NotFound(_) | Self::UserWithEmailExists => BoxBody::new(format!("{{\"error\": \"{self}\"}}")),
            Self::New(err) => err.error_response().into_body()
        };
        builder.body(body)
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

impl From<DomainError> for Error {
    fn from(err: DomainError) -> Self {
        Self::New(Box::new(err))
    }
}


impl From<Error> for DomainError {
    fn from(err: Error) -> Self {
        DomainError::New(err.into())
    }
}