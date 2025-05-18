#[cfg(feature = "dynamodb")]
use aws_sdk_dynamodb::{primitives::Blob, types::AttributeValue};
use serde::{Serialize, Deserialize, Serializer};
use std::ops::{Deref, DerefMut};
use super::ConversionError;
use bson::oid::ObjectId;
use std::str::FromStr;


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
    type Error = ConversionError;

    fn try_from(id: String) -> Result<Self, Self::Error> {
        ObjectId::from_str(&id).map(Id).map_err(|_| ConversionError::CouldNotConvertStringToID)
    }
}


impl From<Id> for Vec<u8> {
    fn from(id: Id) -> Self {
        id.0.bytes().to_vec()
    }
}


impl Serialize for Id {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
        where
            S: Serializer {
        serializer.serialize_str(&self.0.to_hex())
    }
}


#[cfg(feature = "dynamodb")]
impl From<Id> for Blob {
    fn from(id: Id) -> Self {
        Blob::new(id)
    }
}


#[cfg(feature = "dynamodb")]
impl From<Id> for AttributeValue {
    fn from(value: Id) -> Self {
        AttributeValue::B(value.into())
    }
}


#[cfg(feature = "dynamodb")]
impl TryFrom<Blob> for Id {
    type Error = ConversionError;

    fn try_from(value: Blob) -> Result<Self, Self::Error> {
        let bytes = TryInto::<[u8; 12]>::try_into(value.into_inner()).map_err(|_|ConversionError::CouldNotConvertBlobToID)?;
        let id = ObjectId::from_bytes(bytes);
        Ok(Id(id))
    }
}


#[cfg(feature = "dynamodb")]
impl TryFrom<AttributeValue> for Id {
    type Error = ConversionError;

    fn try_from(value: AttributeValue) -> Result<Self, Self::Error> {
        match value {
            AttributeValue::B(blob) => blob.try_into(),
            AttributeValue::S(string) => string.try_into(),
            _ => Err(ConversionError::UnexpectedDataType("id"))
        }
    }
}
