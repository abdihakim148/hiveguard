#[cfg(feature = "http")]
use actix_web::{Responder, web::Json, http::{Method, StatusCode}};
use crate::ports::outputs::database::Item;
use serde::{Serialize, Deserialize};
use super::{Id, Permission, Error};
use std::str::FromStr;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Scope(
    /// This would be the Id of the service where the resource belongs to
    Id,
    /// This would be the name of the resource.
    String,
    /// This would be the permission of the scope.
    Permission
);


impl From<Scope> for String {
    fn from(scope: Scope) -> Self {
        format!("{}:{}:{}", scope.0.to_hex(), scope.1, scope.2)
    }
}


impl FromStr for Scope {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let splits = s.split(':').collect::<Vec<&str>>();
        if splits.len() != 3 {
            return  Err(Error::conversion_error(Some("invalid scope")));
        }
        let (id, name, permission) = (splits[0], splits[1], splits[2]);
        Ok(Scope(id.parse()?, name.to_string(), permission.parse()?))
    }
}


#[cfg(feature = "http")]
impl Responder for Scope {
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


impl Item for Scope {
    const NAME: &'static str = "scope";
    /// This is the id of the Owner of the scope.
    type PK = Id;
    /// This is the name of the scope.
    type SK = String;
}