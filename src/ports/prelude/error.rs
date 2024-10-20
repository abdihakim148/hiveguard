use std::error::Error as StdError;
#[cfg(feature = "actix")]
use actix_web::ResponseError;

#[cfg(not(feature = "actix"))]
pub trait Error: StdError {}
#[cfg(not(feature = "actix"))]
impl<T: StdError + Clone> Error for T {}

#[cfg(feature = "actix")]
pub trait Error: StdError + ResponseError {}
#[cfg(feature = "actix")]
impl<T: StdError + ResponseError> Error for T {}