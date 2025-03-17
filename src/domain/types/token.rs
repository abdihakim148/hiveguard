use actix_web::http::StatusCode;
use serde::{Serialize, Deserialize};
use std::{collections::HashMap, slice::Windows};
use super::{Id, Value};
use chrono::{Utc, DateTime};

#[cfg(feature = "http")]
use actix_web::{
    Responder,
    HttpResponse,
    HttpResponseBuilder,
    web::{Json, Data},
    body::BoxBody,
    cookie::Cookie,
    ResponseError
};
// use crate::adaptors::outputs::{database::memory::Memory, mailer::smtp::SmtpMailer};
use std::sync::Arc;
use crate::{
    domain::types::Config,
    ports::ErrorTrait,
    domain::services::Paseto
};
use serde_json::json;


#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Default)]
#[serde(untagged)]
pub enum Audience {
    #[default]
    None,
    One(String),
    Many(Vec<String>)
}


#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(default)]
pub struct Token<T = HashMap<String, Value>> {
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
    #[serde(flatten)]
    pub claims: T
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