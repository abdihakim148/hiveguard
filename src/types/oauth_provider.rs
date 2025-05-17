use serde::{Serialize, Deserialize};
use super::Error;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum OAuthProvider {
    Github
}


impl TryFrom<String> for OAuthProvider {
    type Error = Error;

    fn try_from(provider: String) -> Result<Self, Self::Error> {
        match provider.to_lowercase().as_str() {
            "github" => Ok(OAuthProvider::Github),
            _ => Err(Error::UnsupportedOAuthProvider(provider.to_string())),
        }
    }
}