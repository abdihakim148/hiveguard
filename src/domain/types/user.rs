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
    #[serde(alias = "user_name")]
    pub username: String,
    /// The first name of the user.
    pub first_name: String,
    /// The last name of the user.
    pub last_name: String,
    /// The email address of the user.
    #[serde(flatten)]
    pub contact: Contact,
    /// Authentication method used by the user
    pub login: LoginMethod,
    /// The password of the user.
    #[serde(skip_serializing_if = "is_default")]
    pub password: String,
}

impl User {
    pub const FIELDS: [&str; 9] = ["id", "username", "first_name", "last_name", "email", "email_verified", "phone", "phone_verified", "password"];
    pub fn token(&self, issuer: String, audience: Audience, ttl: i64) -> Token {
        let id = Default::default();
        let subject = self.id;
        let issued_at = Utc::now();
        let not_before = None;
        let expiration = issued_at + Duration::seconds(ttl);
        let claims = Default::default();
        Token{id, issuer, subject, audience, expiration, not_before, issued_at, claims}
    }

    pub fn from_provider(mut map: HashMap<String, Value>, fields: &[Option<String>; Self::FIELDS.len()], provider_name: &str) -> Result<User, Error> {
        let fields = fields.into_iter().enumerate().map(|(index, field)|{
            match field {
                Some(field) => field.as_str(),
                None => Self::FIELDS[index]
            }
        }).collect::<Box<[&str]>>();
        let id = Default::default();
        let username = map.remove(fields[1]).ok_or(Error::internal::<Box<dyn StdError + Send + Sync>>(format!("the provider {provider_name} hasn't provided the username field which was expected as {}", fields[1]).into()))?.try_into()?;
        let first_name = map.remove(fields[2]).ok_or(Error::internal::<Box<dyn StdError + Send + Sync>>(format!("the provider {provider_name} hasn't provided the first_name field which was expected as {}", fields[2]).into()))?.try_into()?;
        let last_name = map.remove(fields[3]).ok_or(Error::internal::<Box<dyn StdError + Send + Sync>>(format!("the provider {provider_name} hasn't provided the last_name field which was expected as {}", fields[3]).into()))?.try_into()?;
        // change contact field names for easier conversion.
        for index in 4..=7 {
            if let Some(value) = map.remove(fields[index]) {
                map.insert(Self::FIELDS[index].to_string(), value);
            }
        }
        let contact = map.try_into().map_err(|err|Error::internal::<Box<dyn StdError + Send + Sync>>(format!("the provider {provider_name} has'nt provided contact details as supposed with error: {err}").into()))?;
        let login = LoginMethod::Social(provider_name.to_string());
        let password = Default::default();
        Ok(Self{id, username, first_name, last_name, contact, login, password})
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
    type SK = Contact;
}
