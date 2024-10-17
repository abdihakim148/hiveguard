#![allow(unused)]
mod conversion;
mod r#type;

use std::fmt;
use thiserror::Error as ThisError;
use std::error::Error as StdError;
use argon2::password_hash::errors::Error as HashError;
use actix_web::{http::StatusCode, ResponseError};
use serde_json::Error as JsonError;
pub use conversion::ConversionError;
pub use r#type::*;

#[derive(Debug, Clone, PartialEq, ThisError)]
pub enum Error {
    #[error("domain: conversion_error: {0}")]
    ConversionError(ConversionError),
    #[error("domain: hashing_error: {0}")]
    HashingError(HashError),
}


impl From<HashError> for Error {
    fn from(err: HashError) -> Self {
        Self::HashingError(err)
    }
}