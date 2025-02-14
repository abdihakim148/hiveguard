use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use super::{Id, Value};
use chrono::{Utc, DateTime};

#[cfg(feature = "http")]
use actix_web::{
    Responder,
    HttpResponse,
    web::Json,
    cookie::Cookie,
    body::BoxBody,
    ResponseError
};
use serde_json::json;
use crate::{domain::services::Paseto, ports::ErrorTrait};


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

#[cfg(feature = "http")]
impl Responder for Token {
    type Body = BoxBody;

    fn respond_to(self, req: &actix_web::HttpRequest) -> actix_web::HttpResponse<Self::Body> {
        // Check Accept header
        let is_json = req.headers().get("Content-Type")
            .map(|h| h.to_str().unwrap_or(""))
            .map(|h| h.contains("json"))
            .unwrap_or(false);

        // Try to sign the token
        match self.try_sign(&Default::default()) {  // Provide a default PasetoKeys
            Ok(signed_token) => {
                if !is_json {
                    // If HTML is requested, set as cookie
                    HttpResponse::Ok()
                        .cookie(Cookie::new("token", signed_token.clone()))
                        .body(BoxBody::new(""))
                } else {
                    // Set token in Authorization header for JSON
                    HttpResponse::Ok()
                        .insert_header(("Authorization", format!("Bearer {}", signed_token)))
                        .body(BoxBody::new(""))
                }
            },
            Err(err) => {
                // If signing fails, convert to HTTP response
                let status = err.status();
                let body = BoxBody::new(err.user_message());
                HttpResponse::with_body(status, body)
            }
        }
    }
}
