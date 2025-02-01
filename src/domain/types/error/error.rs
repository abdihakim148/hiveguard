#[cfg(feature = "http")]
use actix_web::{http::StatusCode, HttpResponse, body::BoxBody};
use lettre::address::AddressError as EmailAddressError;
use argon2::password_hash::errors::Error as HashError;
use std::fmt::{Display, Formatter, Result, Debug};
use actix_web::{error::ResponseError, Responder};
use lettre::transport::smtp::Error as SmtpError;
use lettre::error::Error as LettreError;
use rusty_paseto::core::PasetoError;
use std::error::Error as StdError;
use serde_json::to_string;
use actix_web::web::Json;
use serde::Serialize;
use std::any::TypeId;
use Error::*;


pub trait ErrorTrait: ResponseError + StdError + Display + Debug {}
impl<T: ResponseError + StdError + Serialize> ErrorTrait for T {}


#[derive(Debug)]
pub enum Error {
    NotFound(&'static str),
    Conflict(&'static str),
    Internal(Box<dyn StdError>),
    InvalidEmailAddress,
    InvalidToken,
    ExpiredToken,
    /// expected type, found type, field name, http status code
    ConversionError(TypeId, TypeId, Option<&'static str>, u16),
    #[cfg(feature = "http")]
    Custom(Box<dyn ErrorTrait>)
}


impl Error {
    fn msg(&self) -> String {
        match self {
            NotFound(name) => format!("{} not found", name),
            Conflict(name) => format!("{} already exists", name),
            Internal(_) => String::from("internal server error"),
            InvalidEmailAddress => String::from("invalid email address"),
            InvalidToken => String::from("invalid token"),
            ExpiredToken => String::from("expired token"),
            ConversionError(expected, found, field, status) => {
                match status {
                    500.. => String::from("internal server Error"),
                    _ => format!("{self}")
                }
            },
            Custom(err) => String::new()
        }
    }
}


impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            NotFound(name) => write!(f, "{} not found", name),
            Conflict(name) => write!(f, "{} already exists", name),
            Internal(err) => write!(f, "internal: {}", err),
            InvalidEmailAddress => write!(f, "invalid email address"),
            InvalidToken => write!(f, "invalid token"),
            ExpiredToken => write!(f, "expired token"),
            ConversionError(expected, found, field, _status) => {
                match field {
                    Some(field) => write!(f, "expected {:?} instead got {:?} for field `{}`", expected, found, field),
                    None => write!(f, "expected {:?} instead got {:?}", expected, found),
                }
            },
            Custom(err) => Display::fmt(&self, f)
        }
    }
}


impl StdError for Error {}

impl Serialize for Error {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
        where
            S: serde::Serializer {
        #[derive(Serialize)]
        struct Serializer {
            msg: String
        }

        let ser = Serializer{msg: self.msg()};
        ser.serialize(serializer)
    }
}

#[cfg(feature = "http")]
impl ResponseError for Error {
    fn status_code(&self) -> StatusCode {
        match self {
            NotFound(_) => StatusCode::NOT_FOUND,
            Conflict(_) => StatusCode::CONFLICT,
            Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
            InvalidEmailAddress => StatusCode::BAD_REQUEST,
            InvalidToken => StatusCode::UNAUTHORIZED,
            ExpiredToken => StatusCode::UNAUTHORIZED,
            ConversionError(_, _, _, status) => StatusCode::from_u16(*status).unwrap_or_default(),
            Custom(err) => err.status_code()
        }
    }

    fn error_response(&self) -> HttpResponse<BoxBody> {
        if let Custom(err) = self {
            return  err.error_response();
        }
        let status = self.status_code();
        let json = serde_json::to_string(&self).unwrap_or(String::from("INTERNAL SERVER ERROR"));
        let body = BoxBody::new(json);
        HttpResponse::build(status).content_type("application/json").body(body)
    }
}


impl From<HashError> for Error {
    fn from(err: HashError) -> Self {
        // Self::Internal(Box::new(err))
        todo!()
    }
}


impl From<EmailAddressError> for Error {
    fn from(_: EmailAddressError) -> Self {
        InvalidEmailAddress
    }
}


impl From<LettreError> for Error {
    fn from(err: LettreError) -> Self {
        match err {
            LettreError::Io(err) => Internal(Box::new(err)),
            _ => InvalidEmailAddress
        }
    }
}


impl From<PasetoError> for Error {
    fn from(err: PasetoError) -> Self {
        match err {
            PasetoError::InvalidSignature => InvalidToken,
            _ => Error::Internal(Box::new(err)),
        }
    }
}


impl From<SmtpError> for Error {
    fn from(err: SmtpError) -> Self {
        Self::Internal(Box::new(err))
    }
}