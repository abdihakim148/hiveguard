use crate::domain::types::error::Error;

pub trait Table {
    type Item;
    type Id;

    async fn create(&self, item: &Self::Item) -> Result<Self::Id, Error>;
    async fn read(&self, id: &Self::Id) -> Option<Self::Item>;
    async fn update(&self, item: &Self::Item) -> Result<Self::Id, Error>;
    async fn delete(&self, id: &Self::Id) -> Result<Self::Id, Error>;
}
