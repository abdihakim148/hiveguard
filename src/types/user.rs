use super::{Contact, ConversionError, Id, Login};
#[cfg(feature = "dynamodb")]
use aws_sdk_dynamodb::types::AttributeValue;
use serde::{Deserialize, Serialize};
use crate::create_date_from_map;
use std::collections::HashMap;
use chrono::{DateTime, Utc};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct User {
    pub id: Id,
    pub username: String,
    pub fullname: String,
    #[serde(flatten)]
    pub contact: Contact,
    #[serde(flatten, skip_serializing_if = "Login::is_empty")]
    pub login: Login,
    #[serde(default)]
    pub profile: Option<String>,
    #[serde(default)]
    pub created_at: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::super::{Email, Phone};
    use super::*;

    #[test]
    fn test_user_serialization_and_deserialization() {
        let id = Id::try_from(String::from("000000000000000000000000")).unwrap();
        let username = String::from("username");
        let fullname = String::from("fullname");
        let email = Email::try_from("user@example.com").unwrap();
        let phone = Phone::try_from(String::from("+25478965439")).unwrap();
        let contact = Contact::Both(phone, email);
        let password = String::from("password");
        let login = Login::Password(password);
        let profile = None;
        let created_at = Utc::now();
        let user = User {
            id,
            username,
            fullname,
            contact,
            login,
            profile,
            created_at,
        };

        let serialized = serde_json::to_string(&user).unwrap();
        println!("Serialized User: {}", serialized);
        let deserialized = serde_json::from_str::<User>(&serialized).unwrap();
        assert_eq!(user, deserialized);
    }
}

#[cfg(feature = "dynamodb")]
impl From<User> for HashMap<String, AttributeValue> {
    fn from(user: User) -> Self {
        let mut map = HashMap::new();
        map.insert("id".into(), user.id.into());
        map.insert("username".into(), AttributeValue::S(user.username));
        map.insert("fullname".into(), AttributeValue::S(user.fullname));
        let iter = user.contact.into();
        map.extend::<HashMap<String, AttributeValue>>(iter);
        let iter = user.login.into();
        map.extend::<HashMap<String, AttributeValue>>(iter);
        if let Some(profile) = user.profile {
            map.insert("profile".into(), AttributeValue::S(profile));
        }
        map.insert(
            "created_at".into(),
            AttributeValue::N(user.created_at.timestamp().to_string()),
        );
        map
    }
}

#[cfg(feature = "dynamodb")]
impl TryFrom<HashMap<String, AttributeValue>> for User {
    type Error = ConversionError;
    fn try_from(mut map: HashMap<String, AttributeValue>) -> Result<Self, Self::Error> {
        let id = map
            .remove("id")
            .ok_or(ConversionError::MissingField("id"))?
            .try_into()?;
        let username = match map
            .remove("username")
            .ok_or(ConversionError::MissingField("username"))?
        {
            AttributeValue::S(username) => Ok(username),
            _ => Err(ConversionError::UnexpectedDataType("username")),
        }?;
        let fullname = map
            .remove("fullname")
            .map_or(Ok(String::new()), |value| match value {
                AttributeValue::S(string) => Ok(string),
                _ => Ok::<_, ConversionError>(String::new()),
            })?;
        let contact = Contact::try_from(&mut map)?;
        let login = Login::try_from(&mut map)?;
        let profile = match map.remove("profile") {
            None => None,
            Some(value) => match value {
                AttributeValue::S(profile) => Some(profile),
                _ => return Err(ConversionError::UnexpectedDataType("profile")),
            },
        };
        let created_at = created_at_date_from_map(&mut map)?;
        Ok(User{id,username,fullname,contact,login,profile,created_at,})
    }
}


create_date_from_map!(created_at_date_from_map, "created_at");