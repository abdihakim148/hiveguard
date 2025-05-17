use serde::{Deserialize, Serialize, Serializer};
use lettre::Address;
use super::Error;


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
    type Error = Error;

    fn try_from(data: EmailData<'a>) -> Result<Self, Self::Error> {
        let address: Address = data.email.parse().map_err(|_| Error::InvalidEmailAddress)?;
        if data.email_verified {
            Ok(Email::Verified(address))
        } else {
            Ok(Email::New(address))
        }
    }
}

impl TryFrom<&str> for Email {
    type Error = Error;

    fn try_from(email: &str) -> Result<Self, Self::Error> {
        let address: Address = email.parse().map_err(|_| Error::InvalidEmailAddress)?;
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