use serde::{Serialize, Deserialize};
use super::{Value, Error};
use std::str::FromStr;


#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum GrantType {
    AuthorizationCode,
    Implicit,
    Password,
    ClientCredentials,
}

impl FromStr for GrantType {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "authorization_code" => Ok(GrantType::AuthorizationCode),
            "implicit" => Ok(GrantType::Implicit),
            "password" => Ok(GrantType::Password),
            "client_credentials" => Ok(GrantType::ClientCredentials),
            _ => Err(Error::conversion_error(Some("invalid grant_type")))?,
        }
    }
}


impl TryFrom<Value> for GrantType {
    type Error = Error;
    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value {
            Value::String(string) => string.as_str().parse(),
            _ => Err(Error::conversion_error(Some("invalid grant_type format")))?
        }
    }
}