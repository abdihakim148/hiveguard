use serde::{Serialize, Deserialize};
use std::error::Error as StdError;
use std::str::FromStr;


#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum GrantType {
    AuthorizationCode,
    Implicit,
    Password,
    ClientCredentials,
}

impl FromStr for GrantType {
    type Err = Box<dyn StdError + 'static>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "authorization_code" => Ok(GrantType::AuthorizationCode),
            "implicit" => Ok(GrantType::Implicit),
            "password" => Ok(GrantType::Password),
            "client_credentials" => Ok(GrantType::ClientCredentials),
            _ => Err(format!("'{}' is not a valid grant type", s))?,
        }
    }
}