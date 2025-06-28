use serde::{Serialize, Deserialize};
use serde_json::{Map, Value};
use chrono::{Utc, DateTime};
use super::Id;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(default)]
pub struct Token<CLAIMS = Map<String, Value>> {
    #[serde(rename = "sid")]
    pub session_id: Id,
    #[serde(rename = "jti")]
    pub id: Id,
    #[serde(rename = "iss")]
    pub issuer: String,
    #[serde(rename = "sub")]
    pub subject: Id,
    #[serde(rename = "aud", skip_serializing_if = "Audience::is_empty")]
    pub audience: Audience,
    #[serde(rename = "exp")]
    pub expiration: DateTime<Utc>,
    #[serde(rename = "nbf")]
    pub not_before: Option<DateTime<Utc>>,
    #[serde(rename = "iat")]
    pub issued_at: DateTime<Utc>,
    pub claims: CLAIMS
}


#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Default)]
#[serde(untagged)]
pub enum Audience {
    #[default]
    None,
    One(String),
    Many(Vec<String>)
}


impl Audience {
    pub fn is_empty(&self) -> bool {
        match self {
            Audience::None => true,
            Audience::One(aud) => aud.is_empty(),
            Audience::Many(aud) => aud.is_empty()
        }
    }
}