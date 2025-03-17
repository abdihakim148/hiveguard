use serde::{Serialize, Deserialize};
use std::hash::Hash;
use std::ops::Deref;

#[derive(Debug, Clone, PartialEq, Hash, Serialize, Deserialize, Eq)]
#[serde(untagged)]
pub enum Either<L, R> {
    Left(L),
    Right(R)
}

impl<L: Deref<Target=str>, R: Deref<Target=str>> Either<L, R> {
    pub fn as_str(&self) -> &str {
        match self {
            Either::Left(l) => l.deref(),
            Either::Right(r) => r.deref(),
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