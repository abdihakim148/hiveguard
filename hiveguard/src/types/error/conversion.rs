use std::fmt::{Display, Formatter};
use std::error::Error as StdError;


#[derive(Debug, PartialEq)]
pub enum ConversionError {
    CouldNotConvertBlobToID,
    CouldNotConvertStringToID,
    UnexpectedDataType(&'static str),
    MissingField(&'static str),
    MissingFields(&'static [&'static str]),
    UnsupportedOAuthProvider(String),
    InvalidEmailAddress,
    InvalidPhoneNumber,
}


impl Display for ConversionError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ConversionError::CouldNotConvertBlobToID => write!(f, "Could not convert the provided blob to a valid ID"),
            ConversionError::CouldNotConvertStringToID => write!(f, "Could not convert the provided string to a valid ID"),
            ConversionError::UnexpectedDataType(field) => write!(f, "unexpected data type for field: {}", field),
            ConversionError::MissingField(field) => write!(f, "missing field: {}", field),
            ConversionError::MissingFields(fields) => write!(f, "missing fields: {}", fields.join(", ")),
            ConversionError::UnsupportedOAuthProvider(provider) => write!(f, "unsupported OAuth provider: {}", provider),
            ConversionError::InvalidEmailAddress => write!(f, "Invalid email address"),
            ConversionError::InvalidPhoneNumber => write!(f, "Invalid phone number"),
        }
    }
}


impl StdError for ConversionError {}