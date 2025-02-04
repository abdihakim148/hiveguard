#[cfg(feature = "http")]
use actix_web::{Responder, web::Json, http::{Method, StatusCode}};
use crate::ports::outputs::database::Item;
use super::{EmailAddress, Id, Contact};
use serde::{Deserialize, Serialize};

/// A struct representing an organisation.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Organisation {
    /// The unique identifier for the organisation.
    pub id: Id,
    /// The name of the organisation.
    pub name: String,
    /// The user ids of the founders of the Organisation.
    pub owners: Vec<Id>,
    /// A list of named email addresses associated with the organisation.
    pub emails: Vec<(String, EmailAddress)>,
    /// The domain of the organisation, if available.
    pub domain: Option<String>,
    /// The home URL of the organisation, if available.
    pub home: Option<String>,
    /// A list of named contact information associated with the organisation.
    pub contacts: Vec<(String, Contact)>,
}

#[cfg(feature = "http")]
impl Responder for Organisation {
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

impl Item for Organisation {
    const NAME: &'static str = "organisation";
    const FIELDS: &'static [&'static str] = &["id", "name", "owners", "emails", "domains","home", "contacts"];
    /// This is the Organisation id.
    type PK = Id;
    /// This is the name of the organisation.
    type SK = String;
}
