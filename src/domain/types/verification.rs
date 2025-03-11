#[cfg(feature = "http")]
use actix_web::{Responder, web::Json, http::{Method, StatusCode}};
use crate::ports::outputs::{verify::Code, database::Item};
use serde::{Serialize, Deserialize, de::DeserializeOwned};
use super::{Id, Either, Phone, EmailAddress};
use chrono::{DateTime, Utc, Duration};
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
pub struct Verification<ID = Id> {
    /// This is the owner of the verification code.
    /// In other words. This is the user being verified
    pub owner_contact: Either<Phone, EmailAddress>,
    /// This is the Id of the verification code
    pub id: ID,
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


impl<ID: PartialEq + Clone + std::hash::Hash> Item for Verification<ID> {
    /// This is the owner id
    type PK = Either<Phone, EmailAddress>;
    /// This is the verification id
    type SK = ID;
}


impl<ID: Serialize + DeserializeOwned, T: Clone + Into<Either<Phone, EmailAddress>>> Code<T, 6> for Verification<ID> {
    type Id = ID;
    fn new(contact: &T, ttl: Option<i64>, id: Self::Id) -> Self {
        let seconds = match ttl {Some(secs) => secs, None => 60*5};
        let code = rand::random_range(10000..999999);
        let owner_contact = contact.clone().into();
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