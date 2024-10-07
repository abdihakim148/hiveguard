use std::collections::HashMap;
use std::convert::{TryFrom, TryInto};
use serde::{Serialize, Deserialize};
use super::number::Number;

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
    type Error = &'static str;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value {
            Value::None => Ok(()),
            _ => Err("Invalid conversion to ()"),
        }
    }
}

impl TryFrom<Value> for bool {
    type Error = &'static str;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value {
            Value::Bool(b) => Ok(b),
            _ => Err("Invalid conversion to bool"),
        }
    }
}

impl TryFrom<Value> for String {
    type Error = &'static str;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value {
            Value::String(s) => Ok(s),
            _ => Err("Invalid conversion to String"),
        }
    }
}

impl TryFrom<Value> for HashMap<String, Value> {
    type Error = &'static str;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value {
            Value::Object(o) => Ok(o),
            _ => Err("Invalid conversion to HashMap<String, Value>"),
        }
    }
}

impl<T: TryFrom<Value>> TryFrom<Value> for Vec<T> {
    type Error = &'static str;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value {
            Value::Vec(value) => {
                let mut array = Vec::new();
                let mut iter = value.into_iter();
                while let Some(value) = iter.next() {
                    ///// this if statement has to be replaced with propper error handling.
                    if let Ok(value) = value.try_into() {array.push(value);}
                }
                Ok(array)
            },
            _ => Err("Invalid conversion to Vec<Value>"),
        }
    }
}

impl<T: TryFrom<Number>> TryFrom<Value> for (T,) {
    type Error = &'static str;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value {
            Value::Number(_) => todo!(),
            _ => Err("Invalid conversion from Value"),
        }
    }
}
