use serde::{Serialize, Deserialize, Serializer, Deserializer};
use serde::de::{self, Visitor, MapAccess};
use serde::ser::SerializeStruct;
use std::collections::HashMap;
use super::{Value, Error};
use std::any::TypeId;
use std::ops::Deref;
use std::fmt::{self, Display};


#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Phone {
    New(String),
    Verified(String)
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

    fn verified(&self) -> bool {
        match self {
            Self::New(_) => false,
            Self::Verified(_) => true,
        }
    }
}


impl Serialize for Phone {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("Phone", 2)?;
        match self {
            Phone::New(phone) => {
                state.serialize_field("phone", phone)?;
                state.serialize_field("phone_verified", &false)?;
            }
            Phone::Verified(phone) => {
                state.serialize_field("phone", phone)?;
                state.serialize_field("phone_verified", &true)?;
            }
        }
        state.end()
    }
}


impl<'de> Deserialize<'de> for Phone {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {

        struct PhoneVisitor;

        impl<'de> Visitor<'de> for PhoneVisitor {
            type Value = Phone;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct Phone")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
                where
                    E: de::Error, {
                if !Phone::valid(v) {
                    return Err(de::Error::custom(Error::InvalidPhone));
                }
                Ok(Phone::New(v.into()))
            }

            fn visit_map<V>(self, mut map: V) -> Result<Phone, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut phone = None;
                let mut phone_verified = None;
                while let Some(key) = map.next_key()? {
                    match key {
                        "phone" => {
                            if phone.is_some() {
                                return Err(de::Error::duplicate_field("phone"));
                            }
                            let phone_str: String = map.next_value()?;
                            if !Phone::valid(&phone_str) {
                                return Err(de::Error::custom("invalid phone number"));
                            }
                            phone = Some(phone_str);
                        }
                        "phone_verified" => {
                            if phone_verified.is_some() {
                                return Err(de::Error::duplicate_field("phone_verified"));
                            }
                            phone_verified = Some(map.next_value()?);
                        },
                        _ => {
                            let _: de::IgnoredAny = map.next_value()?;
                        }
                    }
                }
                let phone = phone.ok_or_else(|| de::Error::missing_field("phone"))?;
                let phone_verified = phone_verified.unwrap_or_default();
                Ok(if phone_verified {
                    Phone::Verified(phone)
                } else {
                    Phone::New(phone)
                })
            }
        }
        deserializer.deserialize_any(PhoneVisitor)
    }
}


impl TryFrom<Value> for Phone {
    type Error = Error;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value {
            Value::String(phone) => {
                if Self::valid(&phone) {
                    return Ok(Phone::New(phone))
                }
                Err(Error::InvalidPhone)?
            },
            Value::Object(map) => map.try_into(),
            _ => Err(Error::InvalidPhone)?
        }
    }
}



impl TryFrom<HashMap<String, Value>> for Phone {
    type Error = Error;
    fn try_from(mut map: HashMap<String, Value>) -> Result<Self, Self::Error> {
        let phone: String = match map.remove("phone") {
            Some(value) => value.try_into()?,
            None => Err(Error::validation("phone", "field not found"))?
        };
        let verified = match map.remove("phone_veified") {
            Some(value) => value.try_into()?,
            None => false
        };
        if !Self::valid(&phone) {
            Err(Error::InvalidPhone)?;
        }
        if verified {
            return Ok(Phone::Verified(phone));
        }
        Ok(Phone::New(phone))
    }
}


impl Deref for Phone {
    type Target = str;
    fn deref(&self) -> &Self::Target {
        match self {
            Self::New(phone) => phone,
            Self::Verified(phone) => phone
        }
    }
}

impl Display for Phone {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::New(phone) => write!(f, "{}", phone),
            Self::Verified(phone) => write!(f, "{}", phone)
        }
    }
}



#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

    #[test]
    fn test_phone_serialization() {
        let phone = Phone::New(String::from("+1234567890"));
        let serialized = serde_json::to_string(&phone).unwrap();
        assert_eq!(serialized, r#"{"phone":"+1234567890","phone_verified":false}"#);

        let phone_verified = Phone::Verified(String::from("+1234567890"));
        let serialized_verified = serde_json::to_string(&phone_verified).unwrap();
        assert_eq!(serialized_verified, r#"{"phone":"+1234567890","phone_verified":true}"#);
    }

    #[test]
    fn test_phone_deserialization() {
        let data = r#"{"phone":"+1234567890","phone_verified":false}"#;
        let deserialized: Phone = serde_json::from_str(data).unwrap();
        assert_eq!(deserialized, Phone::New(String::from("+1234567890")));

        let data_verified = r#"{"phone":"+1234567890","phone_verified":true}"#;
        let deserialized_verified: Phone = serde_json::from_str(data_verified).unwrap();
        assert_eq!(deserialized_verified, Phone::Verified(String::from("+1234567890")));
    }

    #[test]
    fn test_phone_validation() {
        assert!(Phone::valid("+1234567890"));
        assert!(!Phone::valid("123-456-7890"));
        assert!(!Phone::valid("++1234567890"));
        assert!(!Phone::valid("+1234567890+"));
        assert!(!Phone::valid("abcd"));
    }
}
