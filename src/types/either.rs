#[cfg(feature = "dynamodb")]
use aws_sdk_dynamodb::types::AttributeValue;
use super::{ConversionError, Email, Phone};
use serde::{Serialize, Deserialize};
use std::fmt::{Display, Formatter};
use std::collections::HashMap;
use std::hash::Hash;
use std::ops::Deref;


#[derive(Debug, Clone, PartialEq, Hash, Serialize, Deserialize, Eq)]
#[serde(untagged)]
pub enum Either<L, R> {
    Left(L),
    Right(R)
}

impl<T, L: Deref<Target=T>, R: Deref<Target=T>> Deref for Either<L, R> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        match self {
            Either::Left(left) => left.deref(),
            Either::Right(right) => right.deref()
        }
    }
}


impl<'a, T, Y> From<&'a Either<T, Y>> for Either<&'a T, &'a Y> {
    fn from(either: &'a Either<T, Y>) -> Self {
        match either {
            Either::Left(left) => Either::Left(left),
            Either::Right(right) => Either::Right(right)
        }
    }
}


#[cfg(feature = "dynamodb")]
impl<L:Into<AttributeValue>, R:Into<AttributeValue>>  From<Either<L, R>> for AttributeValue {
    fn from(either: Either<L, R>) -> Self {
        match either {
            Either::Left(left) => left.into(),
            Either::Right(right) => right.into()
        }
    }
}


#[cfg(feature = "dynamodb")]
impl<L:Into<HashMap<String, AttributeValue>>, R:Into<HashMap<String, AttributeValue>>> From<Either<L, R>> for HashMap<String, AttributeValue> {
    fn from(either: Either<L, R>) -> Self {
        match either {
            Either::Left(left) => left.into(),
            Either::Right(right) => right.into()
        }
    }
}


impl<T, L: AsRef<T>, R: AsRef<T>> AsRef<T> for Either<L, R> {
    fn as_ref(&self) -> &T {
        match self {
            Either::Left(left) => left.as_ref(),
            Either::Right(right) => right.as_ref()
        }
    }
}

impl<L: Display, R: Display> Display for Either<L, R> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Either::Left(left) => left.fmt(f),
            Either::Right(right) => right.fmt(f)
        }
    }
}


#[cfg(feature = "dynamodb")]
impl<L: TryFrom<AttributeValue, Error = ConversionError>, R: TryFrom<AttributeValue, Error = ConversionError>> TryFrom<AttributeValue> for Either<L, R> {
    type Error = ConversionError;

    fn try_from(value: AttributeValue) -> Result<Self, Self::Error> {
        match L::try_from(value.clone()) {
            Ok(left) => Ok(Either::Left(left)),
            Err(_) => match R::try_from(value) {
                Ok(right) => Ok(Either::Right(right)),
                Err(e) => Err(e)
            }
        }
    }
}


impl TryFrom<&mut HashMap<String, AttributeValue>> for Either<Phone, Email> {
    type Error = ConversionError;

    fn try_from(map: &mut HashMap<String, AttributeValue>) -> Result<Self, Self::Error> {
        match Phone::try_from(&mut *map) {
            Ok(phone) => Ok(Either::Left(phone)),
            Err(_) => match Email::try_from(map) {
                Ok(email) => Ok(Either::Right(email)),
                Err(e) => Err(e)
            }
        }
    }

}