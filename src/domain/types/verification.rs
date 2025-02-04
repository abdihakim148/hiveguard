#[cfg(feature = "http")]
use actix_web::{Responder, web::Json, http::{Method, StatusCode}};
use crate::ports::outputs::database::Item;
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};
use super::Id;


#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize, Default)]
pub enum VerificationMedia {
    #[default]
    Email,
    Whatsapp,
    SMS,
}


#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Default)]
pub struct Verification {
    /// This is the Id of the verification code
    pub id: Id,
    /// This is the owner of the verification code.
    /// In other words. This is the user being verified
    pub owner_id: Id,
    /// This is the actual verification
    pub code: u32,
    /// The media on which the verification code was delivered with.
    pub media: VerificationMedia,
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
    const NAME: &'static str = "verification code";
    const FIELDS: &'static [&'static str] = &["id", "owner_id", "code", "media", "expires"];
    /// This is the verification code id.
    type PK = Id;
    /// This is the owner_id.
    type SK = Id;
}