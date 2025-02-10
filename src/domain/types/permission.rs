use std::fmt::{Display, Formatter, self};
use serde::{Serializer, Deserializer};
use serde::{Serialize, Deserialize};
use serde::de::{self, Visitor};
use super::{Error, Value};
use std::str::FromStr;
use std::any::TypeId;

/// Enum representing various permissions.
#[derive(Copy, Clone, Debug, PartialEq, Default)]
pub enum Permission {
    #[default]
    Read,
    Write,
    Update,
    Delete,
}

impl Serialize for Permission {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let value = match self {
            Permission::Read => 1,
            Permission::Write => 2,
            Permission::Update => 3,
            Permission::Delete => 4,
        };
        serializer.serialize_u8(value)
    }
}

impl<'de> Deserialize<'de> for Permission {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct PermissionVisitor;

        impl<'de> Visitor<'de> for PermissionVisitor {
            type Value = Permission;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("an integer between 1 and 4 representing a Permission")
            }

            fn visit_u8<E>(self, value: u8) -> Result<Permission, E>
            where
                E: de::Error,
            {
                match value {
                    1 => Ok(Permission::Read),
                    2 => Ok(Permission::Write),
                    3 => Ok(Permission::Update),
                    4 => Ok(Permission::Delete),
                    _ => Err(de::Error::custom(format!("invalid permission value: {}", value))),
                }
            }

            fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
                where
                    E: de::Error, {
                Self::visit_u8(self, v as u8)
            }
        }

        deserializer.deserialize_u8(PermissionVisitor)
    }
}


impl Display for Permission {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        use Permission::*;
        match self {
            Read => write!(f, "read"),
            Write => write!(f, "create"),
            Update => write!(f, "update"),
            Delete => write!(f, "delete"),
        }
    }
}


impl FromStr for Permission {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "read" | "1" => Ok(Self::Read),
            "create" | "2" => Ok(Self::Write),
            "update" | "3" => Ok(Self::Update),
            "delete" | "4" => Ok(Self::Delete),
            _ => Err(Error::invalid_format("Permission", s, None))
        }
    }
}


impl TryFrom<Value> for Permission {
    type Error = Error;
    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value {
            Value::String(string) => string.as_str().parse(),
            _ => Err(Error::invalid_format("Permission", format!("{:?}", value), None))
        }
    }
}



#[cfg(test)]
mod tests {
    use serde_json::{to_string, from_str};
    use super::Permission::{self, *};
    #[test]
    fn test_serialization() {
        let read = Read;
        assert_eq!("1", to_string(&read).unwrap());
        let write = Write;
        assert_eq!("2", to_string(&write).unwrap());
        let update = Update;
        assert_eq!("3", to_string(&update).unwrap());
        let delete = Delete;
        assert_eq!("4", to_string(&delete).unwrap());
    }

    #[test]
    fn test_deserialization() {
        let jsons = ["1", "2", "3", "4", "5"];
        let (read, write, update, delete, err) = (from_str(jsons[0]).unwrap(), from_str(jsons[1]).unwrap(), from_str(jsons[2]).unwrap(), from_str(jsons[3]).unwrap(), from_str::<Permission>(jsons[4]));
        assert_eq!((Read, Write, Update, Delete), (read, write, update, delete));
        assert!(err.is_err())
    }
}
