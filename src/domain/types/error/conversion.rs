use super::{super::r#type::Type, GlobalError};
use crate::domain::types::{Number, Value};
#[cfg(feature = "http")]
use actix_web::{body::BoxBody, error::ResponseError, http::StatusCode, HttpResponse as Response};
use serde_json::json;
use std::error::Error as StdError;
use std::fmt::{Debug as DebugTrait, Display, Formatter, Result};

#[derive(Clone, Debug, PartialEq)]
pub struct ConversionError<T: DebugTrait = Value> {
    pub expected: Type,
    pub found: Type,
    pub value: T,
}

impl<T: DebugTrait> ConversionError<T> {
    pub fn new(expected: Type, found: Type, value: T) -> Self {
        Self {
            expected,
            found,
            value,
        }
    }
}

impl<T: DebugTrait> Display for ConversionError<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(
            f,
            "expected: {}. but found: {} of {:?}",
            self.expected, self.found, self.value
        )
    }
}

impl StdError for ConversionError {}

#[cfg(feature = "http")]
impl ResponseError for ConversionError {
    fn status_code(&self) -> StatusCode {
        StatusCode::BAD_REQUEST
    }
    fn error_response(&self) -> Response<BoxBody> {
        let error = format!("{{\"error\": \"{self}\"}}");
        let res = Response::new(self.status_code());
        res.set_body(BoxBody::new(error))
    }
}

impl From<ConversionError> for GlobalError {
    fn from(err: ConversionError) -> Self {
        GlobalError::new(err)
    }
}

impl From<ConversionError<Number>> for ConversionError<Value> {
    fn from(err: ConversionError<Number>) -> Self {
        let value = err.value.into();
        let expected = err.expected;
        let found = err.found;
        ConversionError {
            expected,
            found,
            value,
        }
    }
}
