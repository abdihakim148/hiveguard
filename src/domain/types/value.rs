use std::convert::{TryFrom, TryInto};
use serde::{Serialize, Deserialize};
use crate::domain::types::Error;
use std::collections::HashMap;
use super::number::Number;
use bson::oid::ObjectId;
use std::any::TypeId;
use chrono::Duration;

/// Enum representing various possible object types.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Default)]
#[serde(untagged)]
pub enum Value {
    #[default]
    None,
    Bool(bool),
    Number(Number),
    String(String),
    Object(HashMap<String, Value>),
    Vec(Vec<Value>),
}


impl Value {
    pub fn option<T: TryFrom<Value, Error = Error>>(self) -> Result<Option<T>, Error> {
        match self {
            Self::None => Ok(None),
            _ => Ok(Some(self.try_into()?))
        }
    }
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
        if let Value::None = value {
            Ok(())
        } else {
            Err(Error::invalid_format("()", format!("{:?}", value), None))
        }
    }
}

impl TryFrom<Value> for bool {
    type Error = Error;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        if let Value::Bool(b) = value {
            Ok(b)
        } else {
            Err(Error::invalid_format("bool", format!("{:?}", value), None))
        }
    }
}

impl TryFrom<Value> for String {
    type Error = Error;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        if let Value::String(s) = value {
            Ok(s)
        } else {
            Err(Error::invalid_format("String", format!("{:?}", value), None))
        }
    }
}

impl TryFrom<Value> for HashMap<String, Value> {
    type Error = Error;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        if let Value::Object(o) = value {
            Ok(o)
        } else {
            Err(Error::invalid_format("HashMap<String, Value>", format!("{:?}", value), None))
        }
    }
}

impl<T: TryFrom<Value, Error = Error> + 'static> TryFrom<Value> for Vec<T> {
    type Error = Error;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        if let Value::Vec(value) = value {
            let mut array = Vec::new();
            let mut iter = value.into_iter();
            while let Some(value) = iter.next() {
                array.push(value.try_into()?);
            }
            Ok(array)
        } else {
            Err(Error::invalid_format("Vec<T>", format!("{:?}", value), None))
        }
    }
}

impl<T: TryFrom<Value, Error = Error>, U: TryFrom<Value, Error = Error>> TryFrom<Value> for (T, U) {
    type Error = Error;
    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value {
            Value::Vec(array) => {
                let (t, u) = (array.get(0).cloned().unwrap_or_default(), array.get(1).cloned().unwrap_or_default());
                Ok((t.try_into()?, u.try_into()?))
            },
            _ => Err(Error::invalid_format("Tuple", format!("{:?}", value), None))
        }
    }
}

impl<T: TryFrom<Number, Error = Error> + 'static> TryFrom<Value> for (T,) {
    type Error = Error;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        if let Value::Number(n) = value {
            Ok((n.try_into()?,))
        } else {
            Err(Error::invalid_format("Number", format!("{:?}", value), None))
        }
    }
}


impl TryFrom<Value> for Duration {
    type Error = Error;
    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value {
            Value::Number(number) => number.try_into(),
            _ => Err(Error::invalid_format("Duration", format!("{:?}", value), None))?
        }
    }
}


impl From<&Value> for TypeId {
    fn from(value: &Value) -> Self {
        match value {
            Value::None => TypeId::of::<Option<()>>(),
            Value::Bool(_) => TypeId::of::<bool>(),
            Value::Number(number) => number.into(),
            Value::String(_) => TypeId::of::<String>(),
            Value::Object(_) => TypeId::of::<HashMap<String, Value>>(), 
            Value::Vec(_) => TypeId::of::<Vec<Value>>(),
        }
    }
}
