use super::{super::services::Paseto, Audience, Contact, EmailAddress, Id, Token, Phone, Either};
use crate::ports::outputs::database::Item;
#[cfg(feature = "http")]
use actix_web::{
    http::{Method, StatusCode},
    web::Json,
    Responder,
};
use serde::{Deserialize, Serialize};
use chrono::{Utc, Duration};

/// A struct representing a user.
#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq)]
pub struct User {
    /// The unique identifier for the user.
    #[serde(default)]
    pub id: Id,
    /// The username of the user.
    #[serde(alias = "user_name")]
    pub username: String,
    /// The first name of the user.
    pub first_name: String,
    /// The last name of the user.
    pub last_name: String,
    /// The email address of the user.
    #[serde(flatten)]
    pub email: EmailAddress,
    /// The phone number of the user.
    #[serde(flatten)]
    pub phone: Phone,
    /// The password of the user.
    #[serde(skip_serializing_if = "is_default")]
    pub password: String,
}

impl User {
    pub fn token(&self, issuer: String, audience: Audience, ttl: i64) -> Token {
        let id = Default::default();
        let subject = self.id;
        let issued_at = Utc::now();
        let not_before = None;
        let expiration = issued_at + Duration::seconds(ttl);
        let claims = Default::default();
        let signature = None;
        Token{id, issuer, subject, audience, expiration, not_before, issued_at, claims, signature}
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
    type PK = Id;
    type SK = Either<Phone, EmailAddress>;
}
