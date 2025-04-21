use super::{super::services::Paseto, Contact, Id, Token, Audience, EmailAddress, Phone, Error, Value};
use crate::ports::outputs::database::Item;
use std::error::Error as StdError;
#[cfg(feature = "http")]
use actix_web::{
    http::{Method, StatusCode},
    web::Json,
    Responder,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{Utc, Duration};

/// Authentication method for the user
#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq, Default)]
#[serde(untagged)]
pub enum LoginMethod {
    #[default]
    /// Traditional password-based authentication
    Password,
    /// Social login via OAuth provider
    Social(String), // String represents the provider name
}

/// A struct representing a user.
#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq)]
pub struct User {
    /// The unique identifier for the user.
    #[serde(default)]
    pub id: Id,
    /// The username of the user.
    pub username: String,
    pub name: String,
    /// The email address or phone or both.
    #[serde(flatten)]
    pub contact: Contact,
    /// Authentication method used by the username
    #[serde(default, skip_serializing_if = "is_default")]
    pub login: LoginMethod,
    /// The password of the user.
    #[serde(skip_serializing_if = "is_default")]
    pub password: String,
    pub profile: Option<String>
}

impl User {
    pub fn token(&self, issuer: String, audience: Audience, ttl: i64) -> Token {
        let id = Default::default();
        let subject = self.id;
        let issued_at = Utc::now();
        let not_before = None;
        let expiration = issued_at + Duration::seconds(ttl);
        let claims = Default::default();
        Token{id, issuer, subject, audience, expiration, not_before, issued_at, claims}
    }
}

#[cfg(feature = "http")]
impl Responder for User {
    type Body = <Json<Self> as Responder>::Body;
    fn respond_to(self, req: &actix_web::HttpRequest) -> actix_web::HttpResponse<Self::Body> {
        match req.method() {
            &Method::POST => {
                let mut res = Json(self).respond_to(req);
                *res.status_mut() = StatusCode::CREATED;
                res
            }
            _ => Json(self).respond_to(req),
        }
    }
}

fn is_default<T: Default + PartialEq>(value: &T) -> bool {
    value == &T::default()
}

impl Item for User {
    const NAME: &'static str = "user";
    type PK = Id;
    type SK = Contact;
}
