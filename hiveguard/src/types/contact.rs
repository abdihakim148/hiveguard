use serde::{Deserialize, Deserializer, Serialize, Serializer, de};
#[cfg(feature = "dynamodb")]
use aws_sdk_dynamodb::types::AttributeValue;
use super::{ConversionError, Email, Phone};
use std::collections::HashMap;


#[derive(Debug, Clone, PartialEq)]
pub enum Contact {
    Phone(Phone),
    Email(Email),
    Both(Phone, Email),
}



impl Serialize for Contact {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        #[derive(Serialize)]
        struct ContactData<'a> {
            #[serde(flatten, skip_serializing_if = "Option::is_none")]
            phone: Option<&'a Phone>,
            #[serde(flatten, skip_serializing_if = "Option::is_none")]
            email: Option<&'a Email>,
        }

        let data = match self {
            Contact::Phone(phone) => {
                let phone = Some(phone);
                let email = None;
                ContactData{phone, email}
            },
            Contact::Email(email) => {
                let phone = None;
                let email = Some(email);
                ContactData{phone, email}
            },
            Contact::Both(phone, email) => {
                let phone = Some(phone);
                let email = Some(email);
                ContactData{phone, email}
            }
        };

        data.serialize(serializer)
    }
}


#[cfg(feature = "dynamodb")]
impl From<Contact> for HashMap<String, AttributeValue> {
    fn from(contact: Contact) -> Self {
        match contact {
            Contact::Phone(phone) => phone.into(),
            Contact::Email(email) => email.into(),
            Contact::Both(phone, email) => {
                let mut map: HashMap<String, AttributeValue> = phone.into();
                map.extend::<HashMap<String, AttributeValue>>(email.into());
                map
            }
        }
    }
}


#[cfg(feature = "dynamodb")]
impl TryFrom<&mut HashMap<String, AttributeValue>> for Contact {
    type Error = ConversionError;

    fn try_from(map: &mut HashMap<String, AttributeValue>) -> Result<Self, Self::Error> {
        let email = Email::try_from(&mut *map);
        let phone = Phone::try_from(map);

        match (phone, email) {
            (Ok(phone), Ok(email)) => Ok(Contact::Both(phone, email)),
            (Ok(phone), Err(_)) => Ok(Contact::Phone(phone)),
            (Err(_), Ok(email)) => Ok(Contact::Email(email)),
            (Err(_), Err(_)) => Err(ConversionError::MissingFields(&["phone", "email"])),
        }
    }
}

impl<'de> Deserialize<'de> for Contact {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct ContactData {
            #[serde(flatten)]
            phone: Option<Phone>,
            #[serde(flatten)]
            email: Option<Email>,
        }

        let data = ContactData::deserialize(deserializer)?;

        match (data.phone, data.email) {
            (Some(phone), Some(email)) => Ok(Contact::Both(phone, email)),
            (Some(phone), None) => Ok(Contact::Phone(phone)),
            (None, Some(email)) => Ok(Contact::Email(email)),
            (None, None) => Err(de::Error::custom("Neither phone nor email provided")),
        }
    }
}