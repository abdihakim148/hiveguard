#![allow(unused)]
mod conversion;
mod error;

pub use super::r#type::*;
use super::{Number, Value};
use crate::ports::Error as GlobalError;
#[cfg(feature = "http")]
use actix_web::{
    body::BoxBody, error::ResponseError, http::StatusCode, HttpResponse as Response,
    HttpResponseBuilder as ResponseBuilder,
};
use argon2::password_hash::errors::Error as HashError;
pub use conversion::ConversionError;
use lettre::address::AddressError as EmailAddressError;
use lettre::error::Error as LettreError;
use lettre::transport::smtp::Error as SmtpError;
use rusty_paseto::core::PasetoError;
use serde_json::Error as JsonError;
use std::error::Error as StdError;
use std::fmt::{self, Debug as DebugTrait, Display, Result};
use thiserror::Error as ThisError;

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
    #[error("email error: {0}")]
    MailError(LettreError),
    #[error("smtp error: {0}")]
    SmtpError(SmtpError),
    #[error("{0}")]
    New(GlobalError),
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
            Error::ConversionError(err) => Error::ConversionError(ConversionError {
                expected: err.expected,
                found: err.found,
                value: Value::Number(err.value),
            }),
            Error::EmailAddressError(err) => Error::EmailAddressError(err),
            Error::HashingError(err) => Error::HashingError(err),
            Error::InvalidToken => Error::InvalidToken,
            Error::PasetoError(err) => Error::PasetoError(err),
            Error::InternalServerError(err) => Error::InternalServerError(err),
            Error::MailError(err) => Error::MailError(err),
            Error::SmtpError(err) => Error::SmtpError(err),
            Error::New(err) => Error::New(err),
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

impl From<LettreError> for Error {
    fn from(err: LettreError) -> Self {
        Self::MailError(err)
    }
}

impl From<SmtpError> for Error {
    fn from(err: SmtpError) -> Self {
        Self::SmtpError(err)
    }
}

#[cfg(feature = "http")]
impl<T: DebugTrait> ResponseError for Error<T> {
    fn status_code(&self) -> StatusCode {
        match self {
            Self::ConversionError(_) | Self::EmailAddressError(_) => StatusCode::BAD_REQUEST,
            Self::HashingError(_)
            | Self::PasetoError(_)
            | Self::InternalServerError(_)
            | Self::SmtpError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::InvalidToken => StatusCode::UNAUTHORIZED,
            Self::MailError(err) => match err {
                LettreError::Io(_) => StatusCode::INTERNAL_SERVER_ERROR,
                _ => StatusCode::BAD_REQUEST,
            },
            Self::New(err) => err.status_code(),
        }
    }

    fn error_response(&self) -> Response<BoxBody> {
        let status = self.status_code();
        let mut builder = ResponseBuilder::new(status);
        builder.content_type("application/json");
        let body = match self {
            Self::ConversionError(_) | Self::EmailAddressError(_) => {
                BoxBody::new(format!("{{\"error\": \"{self}\"}}"))
            }
            Self::HashingError(_)
            | Self::PasetoError(_)
            | Self::InternalServerError(_)
            | Self::SmtpError(_) => {
                BoxBody::new(format!("{{\"error\": \"Internal Server Error\"}}"))
            }
            Self::InvalidToken => BoxBody::new(format!("invalid token")),
            Self::MailError(err) => match err {
                LettreError::Io(_) => {
                    BoxBody::new(format!("internal server error. could not send email"))
                }
                _ => BoxBody::new(format!("{}", err)),
            },
            Self::New(err) => err.error_response().into_body(),
        };
        builder.body(body)
    }
}
