use serde::{Serialize, Deserialize};
use super::ConversionError;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum OAuthProvider {
    Github
}


impl TryFrom<String> for OAuthProvider {
    type Error = ConversionError;

    fn try_from(provider: String) -> Result<Self, Self::Error> {
        match provider.to_lowercase().as_str() {
            "github" => Ok(OAuthProvider::Github),
            _ => Err(ConversionError::UnsupportedOAuthProvider(provider)),
        }
    }
}


impl From<OAuthProvider> for String {
    fn from(provider: OAuthProvider) -> Self {
        match provider {
            OAuthProvider::Github => "github".into(),
        }
    }

}