use serde::{Serialize, Deserialize, Serializer, Deserializer};
use serde::de::{self, Visitor, MapAccess};
use serde::ser::SerializeStruct;
use std::fmt;


#[derive(Clone, Debug, PartialEq, Eq)]
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

        const FIELDS: &'static [&'static str] = &["phone", "phone_verified"];
        deserializer.deserialize_struct("Phone", FIELDS, PhoneVisitor)
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