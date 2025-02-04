#[cfg(feature = "http")]
use actix_web::{Responder, web::Json, http::{Method, StatusCode}};
use crate::ports::outputs::database::Item;
use serde::{Deserialize, Serialize};
use super::Id;

/// A struct representing a resource.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Resource {
    /// The unique identifier for the resource.
    pub id: Id,
    /// The unique identifier for the resouce owner.
    /// This might be a user or an organisation or even a service
    pub owner_id: Id,
    /// The name of the resource.
    pub name: String,
    /// The URL of the resource, if available.
    pub url: Option<String>,
}

#[cfg(feature = "http")]
impl Responder for Resource {
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

impl Item for Resource {
    const NAME: &'static str = "resource";
    const FIELDS: &'static [&'static str] = &["id", "owner_id", "name", "url"];
    /// This is the resource id
    type PK = Id;
    /// This is the resource's owner_id
    type SK = Id;
}
