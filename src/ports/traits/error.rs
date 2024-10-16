use std::error::Error as StdError;
use std::fmt::Display;
use actix_web::http::StatusCode;
#[cfg(feature = "actix")]
use actix_web::{error::ResponseError, http::StatusCode, HttpResponse as Response, body::BoxBody, ResponseError};


pub trait Error: StdError + Display {
    #[cfg(feature = "actix")]
    fn status(&self) -> StatusCode;
    #[cfg(feature = "actix")]
    fn reponse(&self) -> Response<BoxBody>;
}


#[cfg(feature = "actix")]
impl<T: Error> ResponseError for T {
    fn status_code(&self) -> StatusCode {
        self.status()
    }

    fn error_response(&self) -> Response<BoxBody> {
        self.response()
    }
}