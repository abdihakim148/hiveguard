#[cfg(feature = "dynamodb")]
use aws_sdk_dynamodb::types::AttributeValue;
use serde::{Serialize, Deserialize};
use super::{ConversionError, Id};
use crate::create_date_from_map;
use std::collections::HashMap;
use chrono::{Utc, DateTime};

// The Session struct represents a user session in the system.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Session {
    pub id: Id,
    pub user_id: Id,
    pub is_active: bool,
    pub refresh_token_id: Id,
    pub previous_refresh_token_id: Option<Id>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}


#[cfg(feature = "dynamodb")]
impl From<Session> for HashMap<String, AttributeValue> {
    fn from(session: Session) -> Self {
        let mut map = HashMap::new();
        map.insert("id".into(), session.id.into());
        map.insert("user_id".into(), session.user_id.into());
        map.insert("is_active".into(), AttributeValue::Bool(session.is_active));
        map.insert("refresh_token_id".into(), session.refresh_token_id.into());
        if let Some(previous_refresh_token_id) = session.previous_refresh_token_id {
            map.insert("previous_refresh_token_id".into(), previous_refresh_token_id.into());
        }
        map.insert("created_at".into(), AttributeValue::N(session.created_at.timestamp().to_string()));
        map.insert("updated_at".into(), AttributeValue::N(session.updated_at.timestamp().to_string()));
        map.insert("expires_at".into(), AttributeValue::N(session.expires_at.timestamp().to_string()));
        map
    }
}


#[cfg(feature = "dynamodb")]
impl TryFrom<HashMap<String, AttributeValue>> for Session {
    type Error = ConversionError;
    fn try_from(mut map: HashMap<String, AttributeValue>) -> Result<Self, Self::Error> {
        let id = map.remove("id").ok_or(ConversionError::MissingField("id"))?.try_into()?;
        let user_id = map.remove("user_id").ok_or(ConversionError::MissingField("user_id"))?.try_into()?;
        let is_active = match map.remove("is_active"){
            Some(AttributeValue::Bool(b)) => Ok(b),
            Some(_) => Err(ConversionError::UnexpectedDataType("is_active")),
            None => Ok(false),
        }?;
        let refresh_token_id = map.remove("refresh_token_id").ok_or(ConversionError::MissingField("refresh_token_id"))?.try_into()?;
        let previous_refresh_token_id = match map.remove("previous_refresh_token_id") {
            Some(value) => Some(value.try_into()?),
            None => None
        };
        let created_at = created_at_date_from_map(&mut map)?;
        let updated_at = updated_at_date_from_map(&mut map)?;
        let expires_at = expires_at_date_from_map(&mut map)?;
        Ok(Self {
            id,
            user_id,
            is_active,
            refresh_token_id,
            previous_refresh_token_id,
            created_at,
            updated_at,
            expires_at,
        })
    }
}


create_date_from_map!(created_at_date_from_map, "created_at");
create_date_from_map!(updated_at_date_from_map, "updated_at");
create_date_from_map!(expires_at_date_from_map, "expires_at");