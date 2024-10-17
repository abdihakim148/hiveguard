use std::error::Error as StdError;


#[cfg(feature = "actix")]
use actix_web::ResponseError;

#[cfg(not(feature = "actix"))]
pub trait Error: StdError {}

#[cfg(feature = "actix")]
pub trait Error: StdError + ResponseError {}