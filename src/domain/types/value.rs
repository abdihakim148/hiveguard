use std::collections::HashMap;
use std::convert::{TryFrom, TryInto};
use bson::oid::ObjectId;
use serde::{Serialize, Deserialize};
use super::number::Number;
use crate::domain::types::{Error, EmailAddress, Type};

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
        if let Value::None = value {
            Ok(())
        } else {
            Err(Error::ConversionError(ConversionError::new(Type::from(&value), Type::Unknown, value)))
        }
    }
}

impl TryFrom<Value> for bool {
    type Error = Error;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        if let Value::Bool(b) = value {
            Ok(b)
        } else {
            Err(Error::ConversionError(ConversionError::new(Type::from(&value), Type::Bool, value)))
        }
    }
}

impl TryFrom<Value> for String {
    type Error = Error;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        if let Value::String(s) = value {
            Ok(s)
        } else {
            Err(Error::ConversionError(ConversionError::new(Type::from(&value), Type::String, value)))
        }
    }
}

impl TryFrom<Value> for HashMap<String, Value> {
    type Error = Error;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        if let Value::Object(o) = value {
            Ok(o)
        } else {
            Err(Error::ConversionError(ConversionError::new(Type::from(&value), Type::Object(Box::new((Type::String, Type::Unknown))), value)))
        }
    }
}

impl<T: TryFrom<Value, Error: std::fmt::Display>> TryFrom<Value> for Vec<T> {
    type Error = Error;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        if let Value::Vec(value) = value {
            let mut array = Vec::new();
            let mut iter = value.into_iter();
            while let Some(value) = iter.next() {
                array.push(value.try_into().map_err(|e| Error::ConversionError(format!("Failed to convert Vec element: {}", e)))?);
            }
            Ok(array)
        } else {
            Err(Error::ConversionError(ConversionError::new(Type::from(&value), Type::Vec(Box::new(Type::Unknown)), value)))
        }
    }
}

impl TryFrom<Value> for ObjectId {
    type Error = Error;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        if let Value::String(s) = value {
            ObjectId::parse_str(&s).map_err(|_| Error::ConversionError(ConversionError::new(Type::String, Type::Unknown, Value::String(s))))
        } else {
            Err(Error::ConversionError(ConversionError::new(Type::from(&value), Type::String, value)))
        }
    }
}

impl<T: TryFrom<Number, Error = Error>> TryFrom<Value> for (T,) {
    type Error = Error;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        if let Value::Number(n) = value {
            Ok((n.try_into()?,))
        } else {
            Err(Error::ConversionError(ConversionError::new(Type::from(&value), Type::Unknown, value)))
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
                    let verified = match map.remove("verified") {
                        Some(value) => value.try_into()?,
                        None => false,
                    };
                    return match verified {
                        true => Ok(EmailAddress::Verified(email.try_into()?)),
                        false => EmailAddress::new(&TryInto::<String>::try_into(email)?),
                    };
                }
                Err(Error::ConversionError(ConversionError::new(Type::Object(Box::new((Type::String, Type::Unknown))), Type::Unknown, value)))
            }
            _ => Err(Error::ConversionError(ConversionError::new(Type::from(&value), Type::String, value))),
        }
    }
}


impl From<&Value> for Type {
    fn from(value: &Value) -> Self {
        match value {
            Value::None => Type::Unknown,
            Value::Bool(_) => Type::Bool,
            Value::Number(number) => Type::from(number),
            Value::String(_) => Type::String,
            Value::Object(map) => {
                if let Some((_, value)) = map.iter().next() {
                    Type::Object(Box::new((Type::String, Type::from(value))))
                }else {
                    Type::Unknown
                }
            }, 
            Value::Vec(array) => {
                let ty = if let Some(value) = array.first() {Type::from(value)} else {Type::Unknown};
                Type::Vec(Box::new(ty))
            },
        }
    }
}
