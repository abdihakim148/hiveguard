#[cfg(feature = "actix")]
use actix_web::{error::ResponseError, http::StatusCode, HttpResponse as Response, body::BoxBody};
use std::fmt::{Display, Debug as DebugTrait, Formatter, Result};
use crate::ports::traits::Error;
use std::error::Error as StdError;
use crate::domain::types::Value;
use super::r#type::Type;
use serde_json::json;


#[derive(Clone, Debug, PartialEq)]
pub struct ConversionError<T: DebugTrait = Value> {
    expected: Type,
    found: Type,
    value: T,
}


impl<T: DebugTrait> ConversionError<T> {
    pub fn new(expected: Type, found: Type, value: T) -> Self {
        Self{expected, found, value}
    }
}


impl Display for ConversionError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "expected: {}. but found: {} of {:?}", self.expected, self.found, self.value)
    }
}



impl StdError for ConversionError {}


#[cfg(feature = "actix")]
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

impl Error for ConversionError {}