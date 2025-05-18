#[cfg(feature = "dynamodb")]
use aws_sdk_dynamodb::types::AttributeValue;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use std::collections::HashMap;
use super::ConversionError;
use std::borrow::Cow;

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
    type Error = ConversionError;

    fn try_from(data: PhoneData<'a>) -> Result<Self, Self::Error> {
        if !Self::valid(&data.phone) {
            return Err(ConversionError::InvalidPhoneNumber);
        }
        if data.phone_verified {
            Ok(Phone::Verified(data.phone.into_owned()))
        } else {
            Ok(Phone::New(data.phone.into_owned()))
        }
    }
}

impl TryFrom<String> for Phone {
    type Error = ConversionError;

    fn try_from(phone: String) -> Result<Self, Self::Error> {
        if !Self::valid(&phone) {
            return Err(ConversionError::InvalidPhoneNumber);
        }
        Ok(Phone::New(phone))
    }
}


#[cfg(feature = "dynamodb")]
impl From<Phone> for HashMap<String, AttributeValue> {
    fn from(phone: Phone) -> Self {
        let data = PhoneData::from(&phone);
        let mut map = HashMap::new();
        map.insert(
            "phone".to_string(),
            AttributeValue::S(data.phone.into_owned()),
        );
        map.insert(
            "phone_verified".to_string(),
            AttributeValue::Bool(data.phone_verified),
        );
        map
    }
}


#[cfg(feature = "dynamodb")]
impl TryFrom<&mut HashMap<String, AttributeValue>> for Phone {
    type Error = ConversionError;

    fn try_from(map: &mut HashMap<String, AttributeValue>) -> Result<Self, Self::Error> {
        let phone = match map.remove("phone") {
            Some(value) => {
                match value {
                    AttributeValue::S(phone) => phone,
                    _ => return Err(ConversionError::UnexpectedDataType("phone"))
                }
            },
            None => return Err(ConversionError::MissingField("phone"))
        };
        let phone_verified = match map.remove("phone_verified") {
            Some(value) => {
                match value {
                    AttributeValue::Bool(value) => value,
                    _ => return Err(ConversionError::UnexpectedDataType("phone_verified"))
                }
            },
            None => false,
        };
        let data = PhoneData {
            phone: Cow::Owned(phone),
            phone_verified,
        };
        data.try_into()
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


impl Display for Phone {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Phone::New(address) => write!(f, "{address}"),
            Phone::Verified(address) => write!(f, "{address}"),
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
        assert_eq!(result.err(), Some(ConversionError::InvalidPhoneNumber));
    }
}