#[cfg(feature = "http")]
use actix_web::{Responder, web::Json, http::{Method, StatusCode}};
use crate::ports::outputs::database::Item;
use super::{Id, Permission, Error, Value};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::str::FromStr;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Default)]
pub struct Scope{
    /// This would be the Id of the service where the resource belongs to
    pub id: Id,
    /// This would be the name of the scope.
    pub name: String,
    /// This would be the permission of the scope.
    pub permission: Permission
}


impl From<Scope> for String {
    fn from(scope: Scope) -> Self {
        format!("{}:{}:{}", scope.id.to_hex(), scope.name, scope.permission)
    }
}


impl FromStr for Scope {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let splits = s.split(':').collect::<Vec<&str>>();
        if splits.len() != 3 {
            return Err(Error::invalid_format("scope", "invalid format", None));
        }
        let (id, name, permission) = (splits[0], splits[1], splits[2]);
        let (id, name, permission) = (id.parse()?, name.to_string(), permission.parse()?);
        Ok(Scope{id, name, permission})
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


impl TryFrom<Value> for Scope {
    type Error = Error;
    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value {
            Value::String(string) => string.as_str().parse(),
            Value::Object(map) => map.try_into(),
            _ => Err(Error::invalid_format("Scope", "invalid data format", None))
        }
    }
}


impl TryFrom<HashMap<String, Value>> for Scope {
    type Error = Error;

    fn try_from(mut map: HashMap<String, Value>) -> Result<Self, Self::Error> {
        let id = map.remove("id").ok_or(Error::validation("id", "missing field"))?.try_into()?;
        let name = map.remove("name").ok_or(Error::validation("name", "missing field"))?.try_into()?;
        let permission = map.remove("permission").ok_or(Error::validation("permission", "missing field"))?.try_into()?;
        Ok(Scope{id, name, permission})
    }
}
