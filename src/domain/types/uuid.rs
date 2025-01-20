use serde::{Serialize, Deserialize, Serializer};
use std::ops::{Deref, DerefMut};
use uuid::Uuid as Uid;
use std::str::FromStr;


#[derive(Clone, Debug, Deserialize, Default, PartialEq)]
pub struct Uuid(Uid);

#[allow(unused)]
impl Uuid {
    pub fn new_v4() -> Self {
        Self(Uid::new_v4())
    }
}


impl Deref for Uuid {
    type Target = Uid;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Uuid {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}


impl FromStr for Uuid {
    type Err = Box<dyn std::error::Error>;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        Ok(Uuid(Uid::from_str(s)?))
    }
}


impl Serialize for Uuid {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
        where
            S: Serializer {
        serializer.serialize_str(&self.0.simple().to_string())
    }
}