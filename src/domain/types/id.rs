use serde::{Serialize, Deserialize, Serializer};
use std::ops::{Deref, DerefMut};
use bson::oid::ObjectId;
use std::str::FromStr;
use super::Error;


#[derive(Clone, Debug, Deserialize, Default, PartialEq, Hash, Eq, Copy)]
pub struct Id(pub ObjectId);



impl Deref for Id {
    type Target = ObjectId;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Id {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}


impl FromStr for Id {
    type Err = Error;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let id = match s.parse() {
            Ok(id) => id,
            Err(_) => Err(Error::conversion_error(Some("invalid id")))?
        };
        Ok(Id(id))
    }
}


impl Serialize for Id {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
        where
            S: Serializer {
        serializer.serialize_str(&self.0.to_hex())
    }
}