mod error;

pub use error::Error as ErrorTrait;
use thiserror::Error as ThisError;
#[cfg(feature = "actix")]
use actix_web::ResponseError;


pub type Result<T> = std::result::Result<T, Error>;


#[derive(Debug, ThisError)]
#[error("{0}")]
pub struct Error(Box<dyn ErrorTrait>);

impl Error {
    pub fn new<T: ErrorTrait + 'static>(err: T) -> Self {
        Self(Box::new(err))
    }
}

#[cfg(feature = "actix")]
impl ResponseError for Error {
    fn status_code(&self) -> actix_web::http::StatusCode {
        self.0.status_code()
    }

    fn error_response(&self) -> actix_web::HttpResponse<actix_web::body::BoxBody> {
        self.0.error_response()
    }
}