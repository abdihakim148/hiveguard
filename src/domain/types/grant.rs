use bson::oid::ObjectId;
use super::Permission;
use serde::{Serialize, Deserialize, Serializer, Deserializer};
use serde::de::{self, Visitor};
use std::fmt;
use std::str::FromStr;

#[derive(Clone, Debug, PartialEq)]
pub struct Grant(pub ObjectId, pub Permission);

impl Serialize for Grant {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let (object_id, permission) = (&self.0, &self.1);
        let s = format!("{}:{}", object_id.to_hex(), permission);
        serializer.serialize_str(&s)
    }
}

impl<'de> Deserialize<'de> for Grant {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct GrantVisitor;

        impl<'de> Visitor<'de> for GrantVisitor {
            type Value = Grant;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a string in the format 'rource_id:permission'")
            }

            fn visit_str<E>(self, value: &str) -> Result<Grant, E>
            where
                E: de::Error,
            {
                let parts: Vec<&str> = value.split(':').collect();
                if parts.len() != 2 {
                    return Err(de::Error::invalid_value(de::Unexpected::Str(value), &self));
                }
                let object_id = ObjectId::from_str(parts[0]).map_err(de::Error::custom)?;
                let permission = parts[1].parse().map_err(de::Error::custom)?;
                Ok(Grant(object_id, permission))
            }
        }

        deserializer.deserialize_str(GrantVisitor)
    }
}



#[cfg(test)]
mod tests {
    use serde_json::{to_string, from_str};
    use super::{Grant, super::Permission};
    use bson::oid::ObjectId;


    #[test]
    fn test_serialization() {
        let id = ObjectId::default();
        let id_str = id.to_hex();
        let permission = Permission::Delete;
        let grant = Grant(id, permission);
        let json = to_string(&grant).unwrap();
        assert_eq!(format!("\"{id_str}:delete\""), json)
    }

    #[test]
    fn test_deserialization() {
        let json = "\"000000000000000000000000:update\"";
        let grant = from_str::<Grant>(json).unwrap();
        let id = ObjectId::from_bytes([0u8; 12]);
        let permission = Permission::Update;
        assert_eq!(Grant(id, permission), grant);
    }
}