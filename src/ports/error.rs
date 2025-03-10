use std::error::Error as StdError;
use std::fmt::{Debug, Display};
use serde::Serialize;
use log::error;

#[cfg(feature = "http")]
use actix_web::{
    http::StatusCode,
    HttpResponse,
    body::BoxBody
};

/// Core error trait that all error types must implement
pub trait ErrorTrait: StdError + Debug + Send + Sync + 'static {
    /// Get detailed error information for logging
    fn log_message(&self) -> String;
    
    /// Get user-safe error message
    fn user_message(&self) -> String;
    
    #[cfg(feature = "http")]
    fn status(&self) -> StatusCode;
}

/// Concrete error type that can wrap any ErrorTrait implementation
#[derive(Debug)]
pub struct Error {
    source: Box<dyn ErrorTrait>,
}

impl Error {
    pub fn new<E: ErrorTrait>(error: E) -> Self {
        Self {
            source: Box::new(error)
        }
    }

    #[cfg(feature = "http")]
    pub fn response(&self) -> HttpResponse<BoxBody> {
        let status = self.source.status();
        let body = serde_json::json!({
            "error": self.source.user_message(),
        });
        if status.is_server_error() {
            let msg = self.source.log_message();
            error!("{msg}")
        }
        HttpResponse::build(status).json(body)
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.source.user_message())
    }
}

impl StdError for Error {}

// Add method to get the underlying error source
impl Error {
    pub fn get_source(&self) -> &dyn ErrorTrait {
        self.source.as_ref()
    }
}

impl<T: ErrorTrait> From<T> for Error {
    fn from(error: T) -> Self {
        Self::new(error)
    }
}

impl From<Box<dyn ErrorTrait>> for Error {
    fn from(source: Box<dyn ErrorTrait>) -> Self {
        Self{source}
    }
}

#[cfg(feature = "http")]
impl actix_web::error::ResponseError for Error {
    fn status_code(&self) -> StatusCode {
        self.source.status()
    }

    fn error_response(&self) -> HttpResponse<BoxBody> {
        self.response()
    }
}

/// Type alias for Results using our Error type
pub type Result<T> = std::result::Result<T, Error>;