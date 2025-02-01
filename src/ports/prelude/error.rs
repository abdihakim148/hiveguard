use std::error::Error as StdError;
#[cfg(feature = "http")]
use actix_web::ResponseError;

#[cfg(not(feature = "http"))]
pub trait Error: StdError {}
#[cfg(not(feature = "http"))]
impl<T: StdError + Clone> Error for T {}

#[cfg(feature = "http")]
pub trait Error: StdError + ResponseError {}
#[cfg(feature = "http")]
impl<T: StdError + ResponseError> Error for T {}