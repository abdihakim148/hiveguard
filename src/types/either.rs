use serde::{Serialize, Deserialize};
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