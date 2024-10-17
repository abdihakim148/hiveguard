use crate::domain::types::{Error, Value, ConversionError, Type};
use serde::ser::SerializeStruct;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde::de::{self, MapAccess, Visitor};
use std::fmt;

/// An enum representing the state of an email.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum EmailAddress {
    New(String),
    Verified(String),
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
    pub fn new(email: &str) -> Result<Self, Error> {
        if email.contains('@') && email.contains('.') {
            Ok(EmailAddress::New(email.to_string()))
        } else {
            Err(Error::InvalidEmailAddress)
        }
    }
}

impl Serialize for EmailAddress {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            EmailAddress::New(email) => serializer.serialize_str(email),
            EmailAddress::Verified(email) => {
                let mut state = serializer.serialize_struct("EmailAddress", 2)?;
                state.serialize_field("email", email)?;
                state.serialize_field("verified", &true)?;
                state.end()
            }
        }
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
                formatter.write_str("a string or a map with an email and verified status")
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                EmailAddress::new(value)
                    .map_err(|_| de::Error::custom(Error::InvalidEmailAddress.to_string()))
            }

            fn visit_map<M>(self, mut map: M) -> Result<Self::Value, M::Error>
            where
                M: MapAccess<'de>,
            {
                let mut email = None;
                let mut verified = None;
                while let Some(key) = map.next_key()? {
                    match key {
                        "email" => {
                            if email.is_some() {
                                return Err(de::Error::duplicate_field("email"));
                            }
                            email = Some(map.next_value()?);
                        }
                        "verified" => {
                            if verified.is_some() {
                                return Err(de::Error::duplicate_field("verified"));
                            }
                            verified = Some(map.next_value()?);
                        }
                        _ => {
                            let _: de::IgnoredAny = map.next_value()?;
                        }
                    }
                }
                let email: String = email.ok_or_else(|| de::Error::missing_field("email"))?;
                let verified: bool = verified.unwrap_or(false);
                if verified {
                    Ok(EmailAddress::Verified(email))
                } else {
                    Ok(EmailAddress::New(email))
                }
            }
        }

        deserializer.deserialize_any(EmailAddressVisitor)
    }
}



impl TryFrom<Value> for EmailAddress {
    type Error = Error<Value>;

    fn try_from(mut value: Value) -> Result<Self, Self::Error> {
        match &mut value {
            Value::String(string) => Ok(EmailAddress::new(&string)?),
            Value::Object(ref mut map) => {
                if let Some(email) = map.remove("email") {
                    let verified = match map.remove("verified") {
                        Some(value) => value.try_into()?,
                        None => false,
                    };
                    return match verified {
                        true => Ok(EmailAddress::Verified(email.try_into()?)),
                        false => EmailAddress::new(&TryInto::<String>::try_into(email)?),
                    };
                }
                Err(Error::ConversionError(ConversionError::new(Type::Object(Box::new((Type::String, Type::Unknown))), Type::New(std::any::TypeId::of::<EmailAddress>()), value)))
            }
            _ => Err(Error::ConversionError(ConversionError::new(Type::from(&value), Type::New(std::any::TypeId::of::<EmailAddress>()), value))),
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
        let result = EmailAddress::new(email);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), EmailAddress::New(email.to_string()));
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
        let result = EmailAddress::new(email);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), Error::InvalidEmailAddress);
    }

    #[test]
    fn test_serialize_new_email() {
        let email = EmailAddress::New("test@example.com".to_string());
        let serialized = serde_json::to_string(&email).unwrap();
        assert_eq!(serialized, "\"test@example.com\"");
    }

    #[test]
    fn test_serialize_verified_email() {
        let email = EmailAddress::Verified("verified@example.com".to_string());
        let serialized = serde_json::to_string(&email).unwrap();
        assert_eq!(serialized, "{\"email\":\"verified@example.com\",\"verified\":true}");
    }

    #[test]
    fn test_deserialize_new_email() {
        let data = "\"test@example.com\"";
        let email: EmailAddress = serde_json::from_str(data).unwrap();
        assert_eq!(email, EmailAddress::New("test@example.com".to_string()));
    }

    #[test]
    fn test_deserialize_verified_email() {
        let data = "{\"email\":\"verified@example.com\",\"verified\":true}";
        let email: EmailAddress = serde_json::from_str(data).unwrap();
        assert_eq!(email, EmailAddress::Verified("verified@example.com".to_string()));
    }
}