#[cfg(feature = "http")]
use actix_web::{Responder, web::Json, http::{Method, StatusCode}};
use crate::ports::outputs::database::Item;
use serde::{Deserialize, Serialize};
use bson::oid::ObjectId;
use super::Grant;

/// A struct representing a role with specific permissions on resources.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Role {
    /// The unique identifier for the role.
    pub id: ObjectId,
    /// This is the role owner.
    /// it might be a user, an organisation or a service.
    pub owner_id: ObjectId,
    /// The name of the role
    pub name: String,
    /// The list of resources and their associated permissions.
    pub grants: Vec<Grant>,
}

#[cfg(feature = "http")]
impl Responder for Role {
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

impl Item for Role {
    const NAME: &'static str = "role";
    /// This is the role id
    type PK = ObjectId;
    /// This is the Role's owner_id
    type SK = ObjectId;
}
