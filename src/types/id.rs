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


impl TryFrom<String> for Id {
    type Error = Error;

    fn try_from(id: String) -> Result<Self, Self::Error> {
        ObjectId::from_str(&id).map(Id).map_err(|_| Error::InvalidId(id))
    }
}


impl Serialize for Id {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
        where
            S: Serializer {
        serializer.serialize_str(&self.0.to_hex())
    }
}
