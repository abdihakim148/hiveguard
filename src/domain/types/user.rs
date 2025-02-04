use super::{Contact, Id};
use crate::ports::outputs::database::Item;
#[cfg(feature = "http")]
use actix_web::{
    http::{Method, StatusCode},
    web::Json,
    Responder,
};
use serde::{Deserialize, Serialize};

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
    /// The password of the user.
    #[serde(skip_serializing_if = "is_default")]
    pub password: String,
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
    const FIELDS: &'static [&'static str] = &["id", "user_name", "username", "first_name", "last_name", "conatct", "email", "phone"];
    type PK = Id;
    type SK = Contact;
}
