#[cfg(feature = "dynamodb")]
use aws_sdk_dynamodb::types::AttributeValue;
use super::{OAuthProvider, ConversionError};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum Login {
    #[serde(rename = "password")]
    Password(String),
    #[serde(rename = "oauth")]
    OAuth(OAuthProvider),
}

impl Login {
    // pub fn clear_password(&mut self) {
    //     if let Login::Password(ref mut password) = self {
    //         *password = String::new();
    //     }
    // }

    pub fn is_empty(&self) -> bool {
        match self {
            Login::Password(ref password) => password.is_empty(),
            Login::OAuth(_) => false,
        }
    }
}


#[cfg(feature = "dynamodb")]
impl From<Login> for HashMap<String, AttributeValue> {
    fn from(login: Login) -> Self {
        let mut map = HashMap::new();
        match login {
            Login::Password(password) => {
                map.insert("password".to_string(), AttributeValue::S(password));
            },
            Login::OAuth(oauth) => {
                map.insert("oauth".to_string(), AttributeValue::S(oauth.into()));
            },
        }
        map
    }
}


#[cfg(feature = "dynamodb")]
impl TryFrom<&mut HashMap<String, AttributeValue>> for Login {
    type Error = ConversionError;

    fn try_from(map: &mut HashMap<String, AttributeValue>) -> Result<Self, Self::Error> {
        match map.remove("password") {
            Some(password) => {
                match password {
                    AttributeValue::S(password) => Ok(Login::Password(password)),
                    _ => Err(ConversionError::UnexpectedDataType("password"))
                }
            },
            None => match map.remove("oauth") {
                Some(oauth) => {
                    match oauth {
                        AttributeValue::S(oauth) => {
                            let oauth_provider: OAuthProvider = oauth.try_into()?;
                            Ok(Login::OAuth(oauth_provider))
                        },
                        _ => Err(ConversionError::UnexpectedDataType("oauth"))
                    }
                },
                None => Err(ConversionError::MissingFields(&["password", "oauth"]))
            }
        }
        
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_login_serialization() {
        let login = Login::Password("password123".to_string());
        let serialized = serde_json::to_string(&login).unwrap();
        assert_eq!(serialized, r#"{"password":"password123"}"#);
    }

    #[test]
    fn test_login_deserialization() {
        let data = json!({"oauth": "github"});
        let deserialized: Login = serde_json::from_value(data).unwrap();
        assert_eq!(deserialized, Login::OAuth(OAuthProvider::Github));
    }
}