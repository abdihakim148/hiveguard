#[cfg(feature = "http")]
use actix_web::{Responder, web::Json, http::{Method, StatusCode}};
use crate::ports::outputs::{verify::Code, database::Item};
use super::{Id, Contact, Phone, EmailAddress};
use chrono::{DateTime, Utc, Duration};
use serde::{Serialize, Deserialize};
use std::fmt::{Display, Formatter};
use std::rc::Rc;


#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize, Default)]
pub enum VerificationMedia {
    #[default]
    Email,
    Whatsapp,
    SMS,
}


#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Verification {
    /// This is the owner of the verification code.
    /// In other words. This is the user being verified
    pub owner_contact: Contact,
    /// This is the Id of the verification code
    pub id: Id,
    /// This is the actual verification
    pub code: u32,
    /// This the time when the verification code become invalid.
    pub expires: DateTime<Utc>,
}


#[cfg(feature = "http")]
impl Responder for Verification {
    type Body = <Json<Self> as Responder>::Body;
    fn respond_to(self, req: &actix_web::HttpRequest) -> actix_web::HttpResponse<Self::Body> {
        match req.method() {
            &Method::POST => {
                let mut res = Json(self).respond_to(req);
                *res.status_mut() = StatusCode::CREATED;
                res
            },
            &Method::GET => Json(self).respond_to(req),
            _ => Json(self).respond_to(req)
        }
    }
}


impl Item for Verification {
    /// This is the owner id
    type PK = Contact;
    /// This is the verification id
    type SK = Id;
}


impl Code<Phone> for Verification {
    fn new(phone: &Phone, ttl: Option<i64>) -> Self {
        let seconds = match ttl {Some(secs) => secs, None => 60*5};
        let owner_contact = Contact::Phone(phone.clone());
        let id = Id::default();
        let code = rand::random_range(10000..999999);
        let expires = Utc::now() + Duration::seconds(seconds);
        Self{owner_contact, id, code, expires}
    }

    fn code(&self) -> u32 {
        self.code
    }
}


impl Display for VerificationMedia {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Email => write!(f, "email"),
            Self::SMS => write!(f, "sms"),
            Self::Whatsapp => write!(f, "whatsapp")
        }
    }
}