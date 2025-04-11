use crate::domain::types::{Error, Value};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde::de::{self, MapAccess, Visitor};
use serde::ser::SerializeStruct;
use std::collections::HashMap;
use lettre::address::Address;
use std::any::TypeId;
use std::ops::Deref;
use std::fmt::{self, Display};

/// An enum representing the state of an email.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum EmailAddress {
    New(Address),
    Verified(Address),
}

impl EmailAddress {
    /// Creates a new EmailAddress instance after validating the email address format.
    ///
    /// # Arguments
    ///
    /// * `email` - A string slice that holds the email to validate.
    ///
    /// # Returns
    ///
    /// * `Result<Self>` - Returns `Ok(Self)` if the email is valid, `Err(Error)` otherwise.
    pub fn new(email: &str, verified: bool) -> Result<Self, Error> {
        let address: Address = email.parse().map_err(|_| Error::InvalidEmail)?;
        match verified {
            true => Ok(EmailAddress::Verified(address)),
            false => Ok(EmailAddress::New(address))
        }
    }

    pub fn verified(&self) -> bool {
        match &self {
            EmailAddress::New(_) => false,
            EmailAddress::Verified(_) => true,
        }
    }
}


impl Serialize for EmailAddress {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("EmailAddress", 2)?;
        match self {
            EmailAddress::New(email) => {
                state.serialize_field::<str>("email", email.as_ref())?;
                state.serialize_field("email_verified", &false)?;
            },
            EmailAddress::Verified(email) => {
                state.serialize_field::<str>("email", email.as_ref())?;
                state.serialize_field("email_verified", &true)?;
            }
        }
        state.end()
    }
}

impl<'de> Deserialize<'de> for EmailAddress {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct EmailAddressVisitor;

        impl<'de> Visitor<'de> for EmailAddressVisitor {
            type Value = EmailAddress;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a string or a map with an email and email_verified status")
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                EmailAddress::new(value, false).map_err(de::Error::custom)
            }

            fn visit_map<M>(self, mut map: M) -> Result<Self::Value, M::Error>
            where
                M: MapAccess<'de>,
            {
                let mut email = None;
                let mut email_verified = None;
                while let Some(key) = map.next_key()? {
                    match key {
                        "email" => {
                            if email.is_some() {
                                return Err(de::Error::duplicate_field("email"));
                            }
                            email = Some(map.next_value()?);
                        }
                        "email_verified" => {
                            if email_verified.is_some() {
                                return Err(de::Error::duplicate_field("email_verified"));
                            }
                            email_verified = Some(map.next_value()?);
                        }
                        _ => {
                            let _: de::IgnoredAny = map.next_value()?;
                        }
                    }
                }
                let email: String = email.ok_or_else(|| de::Error::missing_field("email"))?;
                let verified: bool = email_verified.unwrap_or(false);
                EmailAddress::new(&email, verified).map_err(de::Error::custom)
            }
        }

        deserializer.deserialize_any(EmailAddressVisitor)
    }
}



impl TryFrom<Value> for EmailAddress {
    type Error = Error;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value {
            Value::String(email) => {
                EmailAddress::new(&email, false)
            }
            Value::Object(map) => Ok(map.try_into()?),
            _ => Err(Error::invalid_format("EmailAddress", format!("{:?}", value), None)),
        }
    }
}


impl TryFrom<HashMap<String, Value>> for EmailAddress {
    type Error = Error;
    fn try_from(mut map: HashMap<String, Value>) -> Result<Self, Self::Error> {
        if let Some(email) = map.remove("email") {
            let verified = match map.remove("email_verified") {
                Some(value) => value.try_into()?,
                None => false,
            };
            let email: String = email.try_into()?;
            return EmailAddress::new(&email, verified)
        }
        let value = Value::Object(map);
        Err(Error::invalid_format("EmailAddress", format!("{:?}", value), None))
    }
}


impl Deref for EmailAddress {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        match self {
            Self::New(address) => address.as_ref(),
            Self::Verified(address) => address.as_ref()
        }
    }
}

impl Display for EmailAddress {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::New(address) => write!(f, "{}", address),
            Self::Verified(address) => write!(f, "{}", address)
        }
    }
}


impl From<EmailAddress> for Address {
    fn from(email: EmailAddress) -> Self {
        match email {
            EmailAddress::New(address) => address,
            EmailAddress::Verified(address) => address
        }
    }
}



#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

    #[test]
    fn test_email_new_valid() {
        let email = "test@example.com";
        let result = EmailAddress::new(email, false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_deserialize_invalid_email() {
        let data = "\"invalid-email\"";
        let result: Result<EmailAddress, _> = serde_json::from_str(data);
        assert!(result.is_err());
    }

    #[test]
    fn test_email_new_invalid() {
        let email = "invalid-email";
        let result = EmailAddress::new(email, false);
        assert!(result.is_err());
    }

    #[test]
    fn test_serialize_new_email() {
        let email = EmailAddress::new("test@example.com", false).unwrap();
        let serialized = serde_json::to_string(&email).unwrap();
        assert_eq!(serialized, "{\"email\":\"test@example.com\",\"email_verified\":false}");
    }

    #[test]
    fn test_serialize_email_verified_email() {
        let email = EmailAddress::Verified("email_verified@example.com".parse().unwrap());
        let serialized = serde_json::to_string(&email).unwrap();
        assert_eq!(serialized, "{\"email\":\"email_verified@example.com\",\"email_verified\":true}");
    }

    #[test]
    fn test_deserialize_new_email() {
        let data = "\"test@example.com\"";
        let email: EmailAddress = serde_json::from_str(data).unwrap();
        assert_eq!(email, EmailAddress::new("test@example.com", false).unwrap());
    }

    #[test]
    fn test_deserialize_email_verified_email() {
        let data = "{\"email\":\"email_verified@example.com\",\"email_verified\":true}";
        let email: EmailAddress = serde_json::from_str(data).unwrap();
        assert_eq!(email, EmailAddress::Verified("email_verified@example.com".parse().unwrap()));
    }
}
