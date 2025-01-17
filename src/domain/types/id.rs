use serde::{Serialize, Deserialize, Serializer};
use std::ops::{Deref, DerefMut};
use bson::oid::ObjectId;
use std::str::FromStr;


#[derive(Clone, Debug, Deserialize, Default, PartialEq)]
pub struct Id(ObjectId);



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
    type Err = Box<dyn std::error::Error>;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        Ok(Id(ObjectId::from_str(s)?))
    }
}


impl Serialize for Id {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
        where
            S: Serializer {
        serializer.serialize_str(&self.0.to_hex())
    }
}