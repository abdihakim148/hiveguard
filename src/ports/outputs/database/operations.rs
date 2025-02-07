use crate::domain::types::{Key, Error};
use super::Item;


pub trait CreateItem<I: Item>: Sized {
    type Error: Into<Error>;
    async fn create_item(&self, item: I) -> Result<I, Self::Error>;
}


pub trait GetItem<I: Item, O: Item = I>: Sized {
    type Error: Into<Error>;
    async fn get_item(&self, key: Key<I::PK, I::SK>) -> Result<Option<O>, Self::Error>;
}


pub trait GetItems<I: Item, O: Item = I>: Sized {
    type Error: Into<Error>;
    async fn get_items(&self, key: Key<I::PK, I::SK>) -> Result<Option<Vec<O>>, Self::Error>;
}


pub trait UpdateItem<I: Item, O: Item = I>: Sized {
    type Error: Into<Error>;
    async fn update_item(&self, key: Key<I::PK, I::SK>) -> Result<Option<O>, Self::Error>;
}


pub trait DeleteItem<I: Item>: Sized {
    type Error: Into<Error>;
    async fn delete_item(&self, key: Key<I::PK, I::SK>) -> Result<(), Self::Error>;
    async fn delete_items(&self, keys: Vec<Key<I::PK, I::SK>>) -> Result<(), Self::Error> {
        for key in keys {
            self.delete_item(key).await?;
        }
        Ok(())
    }
}
