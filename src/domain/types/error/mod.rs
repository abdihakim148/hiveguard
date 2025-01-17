#![allow(unused)]
mod conversion;

#[cfg(feature = "actix")]
use actix_web::{error::ResponseError, http::StatusCode, HttpResponse as Response, body::BoxBody, HttpResponseBuilder as ResponseBuilder};
use std::fmt::{self, Debug as DebugTrait, Display, Result};
use lettre::address::AddressError as EmailAddressError;
use argon2::password_hash::errors::Error as HashError;
use crate::ports::Error as GlobalError;
pub use conversion::ConversionError;
use rusty_paseto::core::PasetoError;
use serde_json::Error as JsonError;
use thiserror::Error as ThisError;
use std::error::Error as StdError;
use super::{Value, Number};
pub use super::r#type::*;

#[derive(Debug, ThisError)]
pub enum Error<T: DebugTrait = Value> {
    #[error("domain: conversion_error: {0}")]
    ConversionError(ConversionError<T>),
    #[error("domain: hashing_error: {0}")]
    HashingError(HashError),
    #[error("domain: email_error: {0}")]
    EmailAddressError(EmailAddressError),
    #[error("invalid token")]
    InvalidToken,
    #[error("paseto error: {0}")]
    PasetoError(PasetoError),
    #[error("internal server error: {0}")]
    InternalServerError(Box<dyn std::error::Error>),
    #[error("{0}")]
    New(GlobalError)
}


impl<T: DebugTrait> From<HashError> for Error<T> {
    fn from(err: HashError) -> Self {
        Self::HashingError(err)
    }
}


impl<T: DebugTrait> From<EmailAddressError> for Error<T> {
    fn from(err: EmailAddressError) -> Self {
        Self::EmailAddressError(err)
    }
}


impl From<Error> for GlobalError {
    fn from(err: Error) -> Self {
        GlobalError::new(err)
    }
}

impl From<GlobalError> for Error {
    fn from(err: GlobalError) -> Self {
        Self::New(err)
    }
}

impl From<Error<Number>> for Error<Value> {
    fn from(err: Error<Number>) -> Self {
        match err {
            Error::ConversionError(err) => Error::ConversionError(ConversionError{expected: err.expected, found: err.found, value: Value::Number(err.value)}),
            Error::EmailAddressError(err) => Error::EmailAddressError(err),
            Error::HashingError(err) => Error::HashingError(err),
            Error::InvalidToken => Error::InvalidToken,
            Error::PasetoError(err) => Error::PasetoError(err),
            Error::InternalServerError(err) => Error::InternalServerError(err),
            Error::New(err) => Error::New(err)
        }
    }
}


impl From<PasetoError> for Error {
    fn from(err: PasetoError) -> Self {
        match err {
            PasetoError::InvalidSignature => Error::InvalidToken,
            _ => Error::PasetoError(err),
        }
    }
}


#[cfg(feature = "actix")]
impl<T: DebugTrait> ResponseError for Error<T> {
    fn status_code(&self) -> StatusCode {
        match self {
            Self::ConversionError(_) | Self::EmailAddressError(_) => StatusCode::BAD_REQUEST,
            Self::HashingError(_) | Self::PasetoError(_) | Self::InternalServerError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::InvalidToken => StatusCode::UNAUTHORIZED,
            Self::New(err) => err.status_code() 
        }
    }

    fn error_response(&self) -> Response<BoxBody> {
        let status = self.status_code();
        let mut builder = ResponseBuilder::new(status);
        builder.content_type("application/json");
        let body = match self {
            Self::ConversionError(_) | Self::EmailAddressError(_) => BoxBody::new(format!("{{\"error\": \"{self}\"}}")),
            Self::HashingError(_) | Self::PasetoError(_) | Self::InternalServerError(_) => BoxBody::new(format!("{{\"error\": \"Internal Server Error\"}}")),
            Self::InvalidToken => BoxBody::new(format!("invalid token")),
            Self::New(err) => err.error_response().into_body()
        };
        builder.body(body)
    }
}
