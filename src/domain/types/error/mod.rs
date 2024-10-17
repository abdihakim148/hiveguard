#![allow(unused)]
mod conversion;
mod r#type;

use std::fmt::{self, Debug as DebugTrait, Display, Result};
use thiserror::Error as ThisError;
use std::error::Error as StdError;
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
    EmailAddressError(EmailAddressError)
}


impl From<HashError> for Error {
    fn from(err: HashError) -> Self {
        Self::HashingError(err)
    }
}


impl From<EmailAddressError> for Error {
    fn from(err: EmailAddressError) -> Self {
        Self::EmailAddressError(err)
    }
}

#[cfg(feature = "actix")]
impl ResponseError for Error {
    fn status_code(&self) -> StatusCode {
        StatusCode::BAD_REQUEST
    }
}
