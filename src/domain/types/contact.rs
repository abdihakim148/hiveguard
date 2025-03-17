use serde::{Serialize, Deserialize, Serializer, Deserializer};
use super::{Phone, EmailAddress, Value, Error, Either};
use serde::de::{self, Visitor, MapAccess};
use serde::ser::SerializeStruct;
use std::collections::HashMap;
use std::ops::Add;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Contact {
    Phone(Phone),
    Email(EmailAddress),
    Both(Phone, EmailAddress)
}


impl Contact {
    #[cfg(all(feature = "email", not(feature = "phone")))]
    pub fn contact(self) -> Result<EmailAddress, Error> {
        match self {
            Contact::Phone(_) => Err(Error::EmailAddressRequired),
            Contact::Email(email) => Ok(email),
            Contact::Both(_, email) => Ok(email)
        }
    }

    #[cfg(all(feature = "email", not(feature = "phone")))]
    pub fn verified(&self) -> Result<bool, Error> {
        match &self {
            Self::Email(email) => Ok(email.verified()),
            Self::Both(_, email) => Ok(email.verified()),
            Self::Phone(_) => Err(Error::ContactFeatureConflict)
        }
    }

    #[cfg(all(feature = "phone", not(feature = "email")))]
    pub fn contact(self) -> Result<Phone, Error> {
        match self {
            Contact::Email(_) => Err(Error::PhoneNumberRequired),
            Contact::Phone(phone) => Ok(phone),
            Contact::Both(phone, _) => Ok(phone)
        }
    }

    #[cfg(all(feature = "phone", not(feature = "email")))]
    pub fn verified(&self) -> Result<bool, Error> {
        match &self {
            Self::Phone(phone) => phone.verified,
            Self::Both(phone, _) => Ok(phone.verified()),
            Self::Email(email) => Err(Error::ContactFeatureConflict),
        }
    }

    #[cfg(all(feature = "phone", feature = "email"))]
    pub fn contact(self) -> Result<Phone, Error> {
        match self {
            Contact::Email(_) => Err(Error::PhoneNumberRequired),
            Contact::Phone(_) => Err(Error::EmailAddressRequired),
            Contact::Both(phone, _) => Ok(phone)
        }
    }

    #[cfg(all(feature = "phone", feature = "email"))]
    pub fn phone_verified(&self) -> Result<bool, Error> {
        match &self {
            Self::Both(phone, _) => Ok(phone.verified()),
            _ => Err(Error::ContactFeatureConflict),
        }
    }

    #[cfg(all(feature = "phone", feature = "email"))]
    pub fn email_verified(&self) -> Result<bool, Error> {
        match &self {
            Self::Both(_, email) => Ok(email.verified()),
            _ => Err(Error::ContactFeatureConflict),
        }
    }

    #[cfg(all(feature = "phone", feature = "email"))]
    pub fn verified(&self) -> Result<bool, Error> {
        match &self {
            Self::Both(phone, email) => Ok(phone.verified() && email.verified()),
            _ => Err(Error::ContactFeatureConflict),
        }
    }
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
            email: Option<&'a EmailAddress>,
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
            email: Option<EmailAddress>,
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


impl Add for Contact {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        match self {
            Self::Phone(phone) => {
                match rhs {
                    Self::Phone(phone) => Self::Phone(phone),
                    Self::Email(email) => Self::Both(phone, email),
                    Self::Both(phone, email) => Self::Both(phone, email)
                }
            },
            Self::Email(email) => {
                match rhs {
                    Self::Email(email) => Self::Email(email),
                    Self::Phone(phone) => Self::Both(phone, email),
                    Self::Both(phone, email) => Self::Both(phone, email)
                }
            },
            Self::Both(phone, email) => {
                match rhs {
                    Self::Phone(phone) => Self::Both(phone, email),
                    Self::Email(email) => Self::Both(phone, email),
                    Self::Both(phone, email) => Self::Both(phone, email)
                }
            }
        }
    }
}


impl TryFrom<Value> for Contact {
    type Error = Error;
    fn try_from(value: Value) -> Result<Self, Self::Error> {
        let result = value.clone().try_into();
        if let Ok(phone) = result {
            match value.try_into() {
                Ok(email) => Ok(Contact::Both(phone, email)),
                Err(_) => Ok(Contact::Phone(phone))
            }
        }else {
            match value.try_into() {
                Ok(email) => Ok(Contact::Email(email)),
                Err(err) => Err(err)?
            }
        }
    }
}


impl TryFrom<HashMap<String, Value>> for Contact {
    type Error = Error;
    fn try_from(map: HashMap<String, Value>) -> Result<Self, Self::Error> {
        Value::Object(map).try_into()
    }
}


impl From<Either<Phone, EmailAddress>> for Contact {
    fn from(either: Either<Phone, EmailAddress>) -> Self {
        match either {
            Either::Left(phone) => Contact::Phone(phone),
            Either::Right(email) => Contact::Email(email)
        }
    }
}



#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

    #[test]
    fn test_contact_serialization() {
        let phone = Phone::New(String::from("+1234567890"));
        let email = EmailAddress::New("user@example.com".parse().unwrap());

        let contact_phone = Contact::Phone(phone.clone());
        let serialized_phone = serde_json::to_string(&contact_phone).unwrap();
        assert_eq!(serialized_phone, r#"{"phone":"+1234567890","phone_verified":false}"#);

        let contact_email = Contact::Email(email.clone());
        let serialized_email = serde_json::to_string(&contact_email).unwrap();
        assert_eq!(serialized_email, r#"{"email":"user@example.com","email_verified":false}"#);

        let contact_both = Contact::Both(phone.clone(), email.clone());
        let serialized_both = serde_json::to_string(&contact_both).unwrap();
        assert_eq!(serialized_both, r#"{"phone":"+1234567890","phone_verified":false,"email":"user@example.com","email_verified":false}"#);
    }

    #[test]
    fn test_contact_deserialization() {
        let data_phone = r#"{"phone":"+1234567890","email":null}"#;
        let deserialized_phone: Contact = serde_json::from_str(data_phone).unwrap();
        assert!(matches!(deserialized_phone, Contact::Phone(_)));

        let data_email = r#"{"phone":null,"email":"user@example.com"}"#;
        let deserialized_email: Contact = serde_json::from_str(data_email).unwrap();
        assert!(matches!(deserialized_email, Contact::Email(_)));

        let data_both = r#"{"phone":"+1234567890","email":"user@example.com"}"#;
        let deserialized_both: Contact = serde_json::from_str(data_both).unwrap();
        assert!(matches!(deserialized_both, Contact::Both(_, _)));
    }
}