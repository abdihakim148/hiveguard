mod verifications;
mod sessions;
mod users;


pub use verifications::VerificationsTable;
pub use sessions::SessionsTable;
pub use users::UsersTable;


use aws_sdk_dynamodb::types::AttributeValue;
use crate::types::ConversionError;
use std::collections::HashMap;
use serde_json::{Map, Value};


fn strings(values: Vec<Value>) -> Result<Vec<String>, ConversionError> {
    let mut strings = Vec::new();
    for value in values {
        match value {
            Value::String(string) => strings.push(string),
            _ => Err(ConversionError::UnexpectedDataType("Unknown. converting a Vec<Value> to a Vec<String> for function strings"))?
        }
    }
    Ok(strings)
}


fn numbers(values: &Vec<Value>) -> Result<Vec<String>, ConversionError> {
    let mut numbers = Vec::new();
    for value in values {
        match value {
            Value::Number(number) => numbers.push(number.to_string()),
            _ => Err(ConversionError::UnexpectedDataType("Unknown. converting a Vec<Value> to a Vec<String> for function numbers"))?
        }
    }
    Ok(numbers)
}


fn map_to_hash_map(map: Map<String, Value>) -> Result<HashMap<String, AttributeValue>, ConversionError> {
    let mut hash_map = HashMap::new();
    for (key, value) in map {
        let value = value_to_attribute_value(value)?;
        hash_map.insert(key, value);
    }
    Ok(hash_map)
}


fn value_to_attribute_value(value: Value) -> Result<AttributeValue, ConversionError> {
    match value {
        Value::String(string) => Ok(AttributeValue::S(string)),
        Value::Number(number) => Ok(AttributeValue::N(number.to_string())),
        Value::Bool(boolean) => Ok(AttributeValue::Bool(boolean)),
        Value::Array(array) => {
            match numbers(&array) {
                Ok(numbers) => Ok(AttributeValue::Ns(numbers)),
                Err(_) => {
                    let strings = strings(array)?;
                    Ok(AttributeValue::Ss(strings))
                }
            }
        },
        Value::Object(object) => {
            let hash_map = map_to_hash_map(object)?;
            Ok(AttributeValue::M(hash_map))
        },
        Value::Null => Ok(AttributeValue::Null(true))
    }
}