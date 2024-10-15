use std::collections::HashMap;
use std::convert::{TryFrom, TryInto};
use bson::oid::ObjectId;
use serde::{Serialize, Deserialize};
use super::number::Number;
use crate::domain::types::{Error, EmailAddress};

/// Enum representing various possible object types.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Value {
    None,
    Bool(bool),
    Number(Number),
    String(String),
    Object(HashMap<String, Value>),
    Vec(Vec<Value>),
}

/// Converts a `bool` into a `Value`.
impl From<bool> for Value {
    fn from(value: bool) -> Self {
        Value::Bool(value)
    }
}

impl<T: Into<Number>> From<T> for Value {
    fn from(value: T) -> Self {
        Value::Number(value.into())
    }
}

impl From<String> for Value {
    fn from(value: String) -> Self {
        Value::String(value)
    }
}

impl From<&str> for Value {
    fn from(value: &str) -> Self {
        Value::String(value.to_string())
    }
}

impl From<HashMap<String, Value>> for Value {
    fn from(value: HashMap<String, Value>) -> Self {
        Value::Object(value)
    }
}

impl<T: Into<Value>> From<Vec<T>> for Value {
    fn from(value: Vec<T>) -> Self {
        let mut array = Vec::new();
        let mut iter = value.into_iter();
        while let Some(value) = iter.next() {
            array.push(value.into());
        }
        Value::Vec(array)
    }
}

impl TryFrom<Value> for () {
    type Error = Error;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value {
            Value::None => Ok(()),
            Value::Bool(_) => Err(Error::ConversionError("Invalid conversion. Expected () but found Bool".into())),
            Value::Number(_) => Err(Error::ConversionError("Invalid conversion. Expected () but found Number".into())),
            Value::String(_) => Err(Error::ConversionError("Invalid conversion. Expected () but found String".into())),
            Value::Object(_) => Err(Error::ConversionError("Invalid conversion. Expected () but found Object".into())),
            Value::Vec(_) => Err(Error::ConversionError("Invalid conversion. Expected () but found Vec".into())),
        }
    }
}

impl TryFrom<Value> for bool {
    type Error = Error;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value {
            Value::Bool(b) => Ok(b),
            Value::None => Err(Error::ConversionError("Invalid conversion. Expected bool but found None".into())),
            Value::Number(_) => Err(Error::ConversionError("Invalid conversion. Expected bool but found Number".into())),
            Value::String(_) => Err(Error::ConversionError("Invalid conversion. Expected bool but found String".into())),
            Value::Object(_) => Err(Error::ConversionError("Invalid conversion. Expected bool but found Object".into())),
            Value::Vec(_) => Err(Error::ConversionError("Invalid conversion. Expected bool but found Vec".into())),
        }
    }
}

impl TryFrom<Value> for String {
    type Error = Error;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value {
            Value::String(s) => Ok(s),
            Value::None => Err(Error::ConversionError("Invalid conversion. Expected String but found None".into())),
            Value::Bool(_) => Err(Error::ConversionError("Invalid conversion. Expected String but found Bool".into())),
            Value::Number(_) => Err(Error::ConversionError("Invalid conversion. Expected String but found Number".into())),
            Value::Object(_) => Err(Error::ConversionError("Invalid conversion. Expected String but found Object".into())),
            Value::Vec(_) => Err(Error::ConversionError("Invalid conversion. Expected String but found Vec".into())),
        }
    }
}

impl TryFrom<Value> for HashMap<String, Value> {
    type Error = Error;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value {
            Value::Object(o) => Ok(o),
            Value::None => Err(Error::ConversionError("Invalid conversion. Expected HashMap<String, Value> but found None".into())),
            Value::Bool(_) => Err(Error::ConversionError("Invalid conversion. Expected HashMap<String, Value> but found Bool".into())),
            Value::Number(_) => Err(Error::ConversionError("Invalid conversion. Expected HashMap<String, Value> but found Number".into())),
            Value::String(_) => Err(Error::ConversionError("Invalid conversion. Expected HashMap<String, Value> but found String".into())),
            Value::Vec(_) => Err(Error::ConversionError("Invalid conversion. Expected HashMap<String, Value> but found Vec".into())),
        }
    }
}

impl<T: TryFrom<Value, Error: std::fmt::Display>> TryFrom<Value> for Vec<T> {
    type Error = Error;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value {
            Value::Vec(value) => {
                let mut array = Vec::new();
                let mut iter = value.into_iter();
                while let Some(value) = iter.next() {
                    array.push(value.try_into().map_err(|e| Error::ConversionError(format!("Failed to convert Vec element: {}", e)))?);
                }
                Ok(array)
            },
            Value::None => Err(Error::ConversionError("Invalid conversion. Expected Vec<Value> but found None".into())),
            Value::Bool(_) => Err(Error::ConversionError("Invalid conversion. Expected Vec<Value> but found Bool".into())),
            Value::Number(_) => Err(Error::ConversionError("Invalid conversion. Expected Vec<Value> but found Number".into())),
            Value::String(_) => Err(Error::ConversionError("Invalid conversion. Expected Vec<Value> but found String".into())),
            Value::Object(_) => Err(Error::ConversionError("Invalid conversion. Expected Vec<Value> but found Object".into())),
        }
    }
}

impl TryFrom<Value> for ObjectId {
    type Error = Error;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value {
            Value::String(s) => ObjectId::parse_str(&s).map_err(|_| Error::ConversionError("Invalid conversion to ObjectId".into())),
            Value::None => Err(Error::ConversionError("Invalid conversion. Expected ObjectId but found None".into())),
            Value::Bool(_) => Err(Error::ConversionError("Invalid conversion. Expected ObjectId but found Bool".into())),
            Value::Number(_) => Err(Error::ConversionError("Invalid conversion. Expected ObjectId but found Number".into())),
            Value::Object(_) => Err(Error::ConversionError("Invalid conversion. Expected ObjectId but found Object".into())),
            Value::Vec(_) => Err(Error::ConversionError("Invalid conversion. Expected ObjectId but found Vec".into())),
        }
    }
}

impl<T: TryFrom<Number, Error = Error>> TryFrom<Value> for (T,) {
    type Error = Error;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value {
            Value::Number(n) => Ok((n.try_into()?,)),
            Value::None => Err(Error::ConversionError("Invalid conversion. Expected tuple but found None".into())),
            Value::Bool(_) => Err(Error::ConversionError("Invalid conversion. Expected tuple but found Bool".into())),
            Value::String(_) => Err(Error::ConversionError("Invalid conversion. Expected tuple but found String".into())),
            Value::Object(_) => Err(Error::ConversionError("Invalid conversion. Expected tuple but found Object".into())),
            Value::Vec(_) => Err(Error::ConversionError("Invalid conversion. Expected tuple but found Vec".into())),
        }
    }
}




impl TryFrom<Value> for EmailAddress {
    
    type Error = Error;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value {
            Value::String(string) => Ok(EmailAddress::new(&string)?),
            Value::Object(mut map) => {
                if let Some(email) = map.remove("email") {
                    let verified = match map.remove("verified") {Some(value) => value.try_into()?, None => false};
                    return match verified {
                        true => Ok(EmailAddress::Verified(email.try_into()?)),
                        false => EmailAddress::new(&TryInto::<String>::try_into(email)?)
                    }
                }
                Err(Error::ConversionError("Invalid conversion. Expected an EmailAddress but found a object that does not have the email field".into()))
            },
            Value::None => Err(Error::ConversionError("Invalid conversion. Expected an EmailAddress but found None".into())),
            Value::Bool(_) => Err(Error::ConversionError("Invalid conversion. Expected an EmailAddress but found a bool".into())),
            Value::Number(_) => Err(Error::ConversionError("Invalid conversion. Expected an EmailAddress but found a Number".into())),
            Value::Vec(_) => Err(Error::ConversionError("Invalid conversion. Expected an EmailAddress but found a Vec".into())),
        }
    }
}