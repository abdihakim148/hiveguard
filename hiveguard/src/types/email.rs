use serde::{Deserialize, Serialize, Serializer};
#[cfg(feature = "dynamodb")]
use aws_sdk_dynamodb::types::AttributeValue;
use std::fmt::{Display, Formatter};
use std::collections::HashMap;
use super::ConversionError;
use lettre::Address;


#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Email {
    New(Address),
    Verified(Address),
}


#[derive(Serialize, Deserialize, Default)]
struct EmailData<'a> {
    email: &'a str,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    email_verified: bool,
}


impl<'a> From<&'a Email> for EmailData<'a> {
    fn from(email: &'a Email) -> Self {
        match email {
            Email::New(address) => EmailData {
                email: address.as_ref(),
                email_verified: false,
            },
            Email::Verified(address) => EmailData {
                email: address.as_ref(),
                email_verified: true,
            },
        }
    }
}


impl<'a> TryFrom<EmailData<'a>> for Email {
    type Error = ConversionError;

    fn try_from(data: EmailData<'a>) -> Result<Self, Self::Error> {
        let address: Address = data.email.parse().map_err(|_| ConversionError::InvalidEmailAddress)?;
        if data.email_verified {
            Ok(Email::Verified(address))
        } else {
            Ok(Email::New(address))
        }
    }
}

impl TryFrom<&str> for Email {
    type Error = ConversionError;

    fn try_from(email: &str) -> Result<Self, Self::Error> {
        let address: Address = email.parse().map_err(|_| ConversionError::InvalidEmailAddress)?;
        Ok(Email::New(address))
    }
}

impl AsRef<str> for Email {
    fn as_ref(&self) -> &str {
        match self {
            Email::New(address) => address.as_ref(),
            Email::Verified(address) => address.as_ref(),
        }
    }
}


impl Serialize for Email {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let data = EmailData::from(self);
        data.serialize(serializer)
    }
}


impl<'de> Deserialize<'de> for Email {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let data = EmailData::deserialize(deserializer)?;
        Email::try_from(data).map_err(serde::de::Error::custom)
    }
}


impl TryFrom<String> for Email {
    type Error = ConversionError;

    fn try_from(email: String) -> Result<Self, Self::Error> {
        let address: Address = email.parse().map_err(|_| ConversionError::InvalidEmailAddress)?;
        Ok(Email::New(address))
    }
}


#[cfg(feature = "dynamodb")]
impl From<Email> for HashMap<String, AttributeValue> {
    fn from(email: Email) -> Self {
        let data = EmailData::from(&email);
        let mut map = HashMap::new();
        map.insert("email".to_string(), AttributeValue::S(data.email.to_string()));
        map.insert("email_verified".to_string(), AttributeValue::Bool(data.email_verified));
        map
    }
}

impl Display for Email {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Email::New(address) => write!(f, "{address}"),
            Email::Verified(address) => write!(f, "{address}"),
        }
    }
}

#[cfg(feature = "dynamodb")]
impl TryFrom<&mut HashMap<String, AttributeValue>> for Email {
    type Error = ConversionError;

    fn try_from(map: &mut HashMap<String, AttributeValue>) -> Result<Self, Self::Error> {
        let email = map.get("email").ok_or(ConversionError::MissingField("email"))?;
        let email = email.as_s().map_err(|_| ConversionError::UnexpectedDataType("email"))?;
        let email_verified = match map.get("email_verified") {
            Some(value) => *value.as_bool().map_err(|_| ConversionError::UnexpectedDataType("email_verified"))?,
            None => false,
        };
        let email_data = EmailData {
            email,
            email_verified,
        };
        Email::try_from(email_data)
    }
}

#[cfg(feature = "dynamodb")]
impl TryFrom<HashMap<String, AttributeValue>> for Email {
    type Error = ConversionError;

    fn try_from(mut map: HashMap<String, AttributeValue>) -> Result<Self, Self::Error> {
        Email::try_from(&mut map)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_email_serialization_and_deserialization() {
        let email = Email::try_from("user@example.com").unwrap();
        let serialized = serde_json::to_string(&email).unwrap();
        let deserialized: Email = serde_json::from_str(&serialized).unwrap();
        assert_eq!(email.as_ref(), deserialized.as_ref());
    }

    #[test]
    #[should_panic(expected = "InvalidEmailAddress")]
    fn test_email_from_invalid_email_data() {
        let email_data = EmailData{email: "invalid-email", email_verified: false};
        Email::try_from(email_data).unwrap();
    }

    #[test]
    fn test_email_from_valid_email_data() {
        let email_str = "user@example.com";
        let email_data = EmailData{email: email_str, email_verified: false};
        let email = Email::try_from(email_data).unwrap();
        assert_eq!(email.as_ref(), email_str);
    }
}