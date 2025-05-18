use super::{ConversionError, Either, Email, Id, Phone};
#[cfg(feature = "dynamodb")]
use aws_sdk_dynamodb::types::AttributeValue;
use serde::{Serialize, Deserialize};
use crate::create_date_from_map;
use std::collections::HashMap;
use chrono::{Utc, DateTime};


#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Verification<ID = Id> {
    pub owner_contact: Either<Phone, Email>,
    pub id: ID,
    pub code: u32,
    pub expires: DateTime<Utc>,
}

#[cfg(feature = "dynamodb")]
impl<ID: Into<AttributeValue>> From<Verification<ID>> for HashMap<String, AttributeValue> {
    fn from(verification: Verification<ID>) -> Self {
        let mut map = HashMap::new();
        map.insert("owner_contact".to_string(), AttributeValue::S(verification.owner_contact.to_string()));
        map.insert("id".to_string(), verification.id.into());
        map.insert("code".to_string(), AttributeValue::N(verification.code.to_string()));
        map.insert("expires".to_string(), AttributeValue::N(verification.expires.timestamp().to_string()));
        map
    }
}


#[cfg(feature = "dynamodb")]
impl TryFrom<HashMap<String, AttributeValue>> for Verification {
    type Error = ConversionError;

    fn try_from(mut map: HashMap<String, AttributeValue>) -> Result<Self, Self::Error> {
        let owner_contact = Either::<Phone, Email>::try_from(&mut map)?;
        let id = map.remove("id").ok_or(ConversionError::MissingField("id"))?.try_into()?;
        let code = match map.remove("code").ok_or(ConversionError::MissingField("code"))?{
            AttributeValue::N(code) => {
                let code: u32 = code.parse().map_err(|_| ConversionError::UnexpectedDataType("code"))?;
                Ok(code)
            },
            _ => Err(ConversionError::UnexpectedDataType("code"))
        }?;
        let expires = expires_date_from_map(&mut map)?;
        Ok(Verification {
            owner_contact,
            id,
            code,
            expires,
        })
    }
}


create_date_from_map!(expires_date_from_map, "expires");