use crate::domain::types::{Error, Type, ConversionError};
use std::convert::{TryFrom, TryInto};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use super::number::Number;
use bson::oid::ObjectId;

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
    type Error = Error<Value>;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        if let Value::None = value {
            Ok(())
        } else {
            Err(Error::ConversionError(ConversionError::new(Type::from(&value), Type::New(std::any::TypeId::of::<()>()), value)))
        }
    }
}

impl TryFrom<Value> for bool {
    type Error = Error<Value>;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        if let Value::Bool(b) = value {
            Ok(b)
        } else {
            Err(Error::ConversionError(ConversionError::new(Type::from(&value), Type::New(std::any::TypeId::of::<bool>()), value)))
        }
    }
}

impl TryFrom<Value> for String {
    type Error = Error<Value>;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        if let Value::String(s) = value {
            Ok(s)
        } else {
            Err(Error::ConversionError(ConversionError::new(Type::from(&value), Type::New(std::any::TypeId::of::<String>()), value)))
        }
    }
}

impl TryFrom<Value> for HashMap<String, Value> {
    type Error = Error<Value>;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        if let Value::Object(o) = value {
            Ok(o)
        } else {
            Err(Error::ConversionError(ConversionError::new(Type::from(&value), Type::New(std::any::TypeId::of::<HashMap<String, Value>>()), value)))
        }
    }
}

impl<T: TryFrom<Value, Error = Error<Value>> + 'static> TryFrom<Value> for Vec<T> {
    type Error = Error<Value>;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        if let Value::Vec(value) = value {
            let mut array = Vec::new();
            let mut iter = value.into_iter();
            while let Some(value) = iter.next() {
                array.push(value.try_into()?);
            }
            Ok(array)
        } else {
            Err(Error::ConversionError(ConversionError::new(Type::from(&value), Type::New(std::any::TypeId::of::<Vec<T>>()), value)))
        }
    }
}

impl TryFrom<Value> for ObjectId {
    type Error = Error<Value>;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        if let Value::String(s) = value {
            ObjectId::parse_str(&s).map_err(|_| Error::ConversionError(ConversionError::new(Type::String, Type::Unknown, Value::String(s))))
        } else {
            Err(Error::ConversionError(ConversionError::new(Type::from(&value), Type::New(std::any::TypeId::of::<ObjectId>()), value)))
        }
    }
}

impl<T: TryFrom<Number, Error = Error> + 'static> TryFrom<Value> for (T,) {
    type Error = Error<Value>;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        if let Value::Number(n) = value {
            Ok((n.try_into()?,))
        } else {
            Err(Error::ConversionError(ConversionError::new(Type::from(&value), Type::New(std::any::TypeId::of::<(T,)>()), value)))
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
