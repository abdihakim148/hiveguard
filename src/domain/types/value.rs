use std::collections::HashMap;
use std::convert::{TryFrom, TryInto};
use serde::{Serialize, Deserialize};
use super::number::Number;
use crate::domain::types::Error;

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
            _ => Err(Error::ConversionError("Invalid conversion to ()".into())),
        }
    }
}

impl TryFrom<Value> for bool {
    type Error = Error;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value {
            Value::Bool(b) => Ok(b),
            _ => Err(Error::ConversionError("Invalid conversion to bool".into())),
        }
    }
}

impl TryFrom<Value> for String {
    type Error = Error;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value {
            Value::String(s) => Ok(s),
            _ => Err(Error::ConversionError("Invalid conversion to String".into())),
        }
    }
}

impl TryFrom<Value> for HashMap<String, Value> {
    type Error = Error;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value {
            Value::Object(o) => Ok(o),
            _ => Err(Error::ConversionError("Invalid conversion to HashMap<String, Value>".into())),
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
            _ => Err(Error::ConversionError("Invalid conversion to Vec<Value>".into())),
        }
    }
}

impl<T: TryFrom<Number, Error = Error>> TryFrom<Value> for (T,) {
    type Error = Error;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value {
            Value::Number(n) => Ok((n.try_into()?,)),
            _ => Err(Error::ConversionError("Invalid conversion from Value".into())),
        }
    }
}
