#[cfg(feature = "http")]
use actix_web::{Responder, web::Json, http::{Method, StatusCode}};
use crate::ports::outputs::database::Item;
use serde::{Serialize, Deserialize};
use super::{Id, GrantType, Scope};
use chrono::Duration;


#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Service {
    pub id: Id,
    pub owner_id: Id,
    pub name: String,
    pub client_secret: String,
    pub redirect_uris: Vec<String>,
    pub scopes: Vec<Scope>,
    pub grant_types: Vec<GrantType>,
    pub token_expiry: Option<Duration>,
}


#[cfg(feature = "http")]
impl Responder for Service {
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


impl Item for Service {
    const NAME: &'static str = "service";
    /// This is the id.
    type PK = Id;
    /// This is the service name.
    type SK = String;
}
