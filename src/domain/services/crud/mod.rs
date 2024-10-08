use crate::domain::types::{Result, Value};
use std::collections::HashMap;

pub trait Crud: Sized {
    type Id;
    type Verification;
    async fn register(&self) -> Result<Self::Id>;
    async fn read(id: &Self::Id) -> Result<Self>;
    async fn patch(id: &Self::Id, value: HashMap<String, Value>) -> Result<Self>;
    async fn update(id: &Self::Id, value: HashMap<String, Value>) -> Result<Self>;
    async fn delete(id: &Self::Id, verification: Self::Verification) -> Result<()>;
}