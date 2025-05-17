use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use super::Error;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Phone {
    New(String),
    Verified(String)
}

#[derive(Serialize, Deserialize, Default)]
struct PhoneData<'a> {
    phone: Cow<'a, str>,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    phone_verified: bool,
}


impl Phone {
    fn valid(phone: &str) -> bool {
        let mut count = 0;
        let pos = phone.find("+");
        
        for i in phone.as_bytes(){
            if i < &48 && i !=&43 || i > &57 && i != &43{
                return false;
            } else {
                if i == &43{
                    count+=1;
                }
            }
            if pos != None{
                if count > 1 || pos.unwrap() != 0{
                    return false;
                }
            }
        }
        true
    }
}


impl<'a> From<&'a Phone> for PhoneData<'a> {
    fn from(phone: &'a Phone) -> Self {
        match phone {
            Phone::New(phone) => PhoneData {
                phone: Cow::Borrowed(phone),
                phone_verified: false,
            },
            Phone::Verified(phone) => PhoneData {
                phone: Cow::Borrowed(phone),
                phone_verified: true,
            },
        }
    }
}


impl<'a> TryFrom<PhoneData<'a>> for Phone {
    type Error = Error;

    fn try_from(data: PhoneData<'a>) -> Result<Self, Self::Error> {
        if !Self::valid(&data.phone) {
            return Err(Error::InvalidPhoneNumber);
        }
        if data.phone_verified {
            Ok(Phone::Verified(data.phone.into_owned()))
        } else {
            Ok(Phone::New(data.phone.into_owned()))
        }
    }
}

impl TryFrom<String> for Phone {
    type Error = Error;

    fn try_from(phone: String) -> Result<Self, Self::Error> {
        if !Self::valid(&phone) {
            return Err(Error::InvalidPhoneNumber);
        }
        Ok(Phone::New(phone))
    }
}


impl Serialize for Phone {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let data = PhoneData::from(self);
        data.serialize(serializer)
    }
}


impl<'de> Deserialize<'de> for Phone {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let data = PhoneData::deserialize(deserializer)?;
        Phone::try_from(data).map_err(serde::de::Error::custom)
    }
}


impl AsRef<str> for Phone {
    fn as_ref(&self) -> &str {
        match self {
            Phone::New(phone) => phone.as_ref(),
            Phone::Verified(phone) => phone.as_ref(),
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_phone_valid() {
        assert!(Phone::valid("+1234567890"));
        assert!(Phone::valid("1234567890"));
        assert!(!Phone::valid("123456789a"));
        assert!(!Phone::valid("++1234567890"));
        assert!(!Phone::valid("+1234567890+"));
        assert!(!Phone::valid("1234567890+"));
        assert!(!Phone::valid("+1234567890+"));
    }

    #[test]
    fn test_phone_serialization_and_deserialization() {
        let phone = Phone::New("+1234567890".to_string());
        let serialized = serde_json::to_string(&phone).unwrap();
        assert_eq!(serialized, r#"{"phone":"+1234567890"}"#);

        let deserialized: Phone = serde_json::from_str(&serialized).unwrap();
        assert_eq!(phone, deserialized);
    }

    #[test]
    fn test_email_from_valid_email_data() {
        let invalid_phone_data = PhoneData {
            phone: "+19113456812".into(),
            phone_verified: false,
        };
        Phone::try_from(invalid_phone_data).unwrap();
    }

    #[test]
    fn test_phone_from_invalid_phone_data() {
        let invalid_phone_data = PhoneData {
            phone: "123456789a".into(),
            phone_verified: false,
        };
        let result = Phone::try_from(invalid_phone_data);
        assert!(result.is_err());
        assert_eq!(result.err(), Some(Error::InvalidPhoneNumber));
    }
}