#[cfg(feature = "http")]
use actix_web::{Responder, web::Json, http::{Method, StatusCode}};
use crate::ports::outputs::{verify::Code, database::Item};
use serde::{Serialize, Deserialize, de::DeserializeOwned};
use super::{Id, Either, Phone, EmailAddress};
use chrono::{DateTime, Utc, Duration};
use std::fmt::{Display, Formatter};
use std::str::FromStr;


#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum VerificationMedia {
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

#[cfg(feature = "email")]
impl<ID: Serialize + DeserializeOwned + FromStr> Code<EmailAddress, 6> for Verification<ID> {
    type Id = ID;
    fn new(email: &EmailAddress, ttl: Option<i64>, id: Self::Id) -> Self {
        let seconds = match ttl {Some(secs) => secs, None => 60*5};
        let code = rand::random_range(10000..999999);
        let owner_contact = Either::Right(email.clone());
        let expires = Utc::now() + Duration::seconds(seconds);
        Self{owner_contact, id, code, expires}
    }

    fn code(&self) -> u32 {
        self.code
    }
}


#[cfg(feature = "phone")]
impl<ID: Serialize + DeserializeOwned> Code<Phone, 6> for Verification<ID> {
    type Id = ID;
    fn new(phone: &Phone, ttl: Option<i64>, id: Self::Id) -> Self {
        let seconds = match ttl {Some(secs) => secs, None => 60*5};
        let code = rand::random_range(10000..999999);
        let owner_contact = Either::Left(phone.clone());
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


impl Default for VerificationMedia {
    fn default() -> Self {
        #[cfg(all(feature = "phone", feature = "email"))]
        return VerificationMedia::SMS;
        #[cfg(all(feature = "phone", not(feature = "email")))]
        return VerificationMedia::SMS;
        #[cfg(all(feature = "email", not(feature = "phone")))]
        VerificationMedia::Email
    }
}


impl From<String> for VerificationMedia {
    fn from(value: String) -> Self {
        let value = value.to_lowercase();
        let value = value.as_str();
        match value {
            "sms" => VerificationMedia::SMS,
            "whatsapp" => VerificationMedia::Whatsapp,
            "email" => VerificationMedia::Email,
            _ => Default::default()
        }
    }
}