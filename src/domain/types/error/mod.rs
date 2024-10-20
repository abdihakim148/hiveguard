#![allow(unused)]
mod conversion;
mod r#type;

use std::fmt::{self, Debug as DebugTrait, Display, Result};
use thiserror::Error as ThisError;
use std::error::Error as StdError;
use crate::ports::Error as GlobalError;
use argon2::password_hash::errors::Error as HashError;
use serde_json::Error as JsonError;
pub use conversion::ConversionError;
use lettre::address::AddressError as EmailAddressError;
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
    EmailAddressError(EmailAddressError),
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


#[cfg(feature = "actix")]
impl<T: DebugTrait> ResponseError for Error<T> {
    fn status_code(&self) -> StatusCode {
        match self {
            Self::ConversionError(_) | Self::EmailAddressError(_) => StatusCode::BAD_REQUEST,
            Self::HashingError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::New(err) => err.status_code() 
        }
    }
}
