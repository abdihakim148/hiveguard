#![allow(unused)]
mod conversion;
mod r#type;

use std::fmt::{self, Debug as DebugTrait, Display, Result};
use thiserror::Error as ThisError;
use std::error::Error as StdError;
use argon2::password_hash::errors::Error as HashError;
use serde_json::Error as JsonError;
pub use conversion::ConversionError;
use lettre::error::Error as EmailError;
pub use r#type::*;
use super::Value;
#[cfg(feature = "actix")]
use actix_web::{error::ResponseError, http::StatusCode, HttpResponse as Response, body::BoxBody};

#[derive(Debug, ThisError)]
pub enum Error<T: DebugTrait = Value> {
    #[error("domain: conversion_error: {0}")]
    ConversionError(ConversionError<T>),
    #[error("domain: hashing_error: {0}")]
    HashingError(HashError),
    #[error("domain: email_error: {0}")]
    EmailError(EmailError)
}


impl From<HashError> for Error {
    fn from(err: HashError) -> Self {
        Self::HashingError(err)
    }
}


impl From<EmailError> for Error {
    fn from(err: EmailError) -> Self {
        Self::EmailError(err)
    }
}

#[cfg(feature = "actix")]
impl ResponseError for Error {
    fn status_code(&self) -> StatusCode {
        match self {
            Self::ConversionError(err) => err.status_code(),
            Self::HashingError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::EmailError(err) => match err {
                EmailError::EmailMissingAt => StatusCode::BAD_REQUEST,
                EmailError::EmailMissingDomain => StatusCode::BAD_REQUEST,
                EmailError::EmailMissingLocalPart => StatusCode::BAD_REQUEST,
                _ => StatusCode::INTERNAL_SERVER_ERROR
            }
        }
    }


    fn error_response(&self) -> Response<BoxBody> {
        match self {
            Self::ConversionError(err) => err.error_response(),
            Self::HashingError(_) => Response::with_body(self.status_code(), BoxBody::new(format!("{{\"error\": \"{self}\"}}"))),
            Self::EmailError(err) => match err {
                EmailError::EmailMissingAt => Response::with_body(self.status_code(), BoxBody::new(format!("{{\"error\": \"{self}\"}}"))),
                EmailError::EmailMissingDomain => Response::with_body(self.status_code(), BoxBody::new(format!("{{\"error\": \"{self}\"}}"))),
                EmailError::EmailMissingLocalPart => Response::with_body(self.status_code(), BoxBody::new(format!("{{\"error\": \"{self}\"}}"))),
                _ => Response::with_body(self.status_code(), BoxBody::new(format!("{{\"error\": \"Internal Server Error\"}}")))
            }
        }
    }
}