#[cfg(feature = "http")]
use actix_web::{http::{Method, StatusCode}, web::Json, Responder};
use crate::ports::outputs::database::Item;
use serde::{Deserialize, Serialize};
use super::Id;

/// A struct representing a member with roles in an organisation.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Default)]
pub struct Member {
    /// The unique identifier for the organisation.
    pub org_id: Id,
    /// The unique identifier for the user.
    pub user_id: Id,
    /// The title of the member.
    pub title: String,
    /// indicates if the user has the role of owner
    pub owner: bool,
    /// The list of role IDs associated with the member.
    pub roles: Vec<Id>,
}

#[cfg(feature = "http")]
impl Responder for Member {
    type Body = <Json<Self> as Responder>::Body;
    fn respond_to(self, req: &actix_web::HttpRequest) -> actix_web::HttpResponse<Self::Body> {
        match req.method() {
            &Method::POST => {
                let mut res = Json(self).respond_to(req);
                *res.status_mut() = StatusCode::CREATED;
                res
            }
            &Method::GET => Json(self).respond_to(req),
            _ => Json(self).respond_to(req),
        }
    }
}

impl Item for Member {
    /// This is the org_id
    type PK = Id;
    /// This is the user_id
    type SK = Id;
}
