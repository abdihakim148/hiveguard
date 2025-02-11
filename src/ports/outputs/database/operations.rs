use crate::domain::types::{Key, Value};
use std::collections::HashMap;
use crate::ports::ErrorTrait;
use super::Item;

/// This trait is used to create a new Item.
pub trait CreateItem<I: Item>: Sized {
    type Error: ErrorTrait;
    /// This method stores an item into the database and returns it if the operation was successfull. else it returns an Error.
    async fn create_item(&self, item: I) -> Result<I, Self::Error>;
}

/// This trait is used to get an item from the database.
/// I: is for the Item on which the trait is implemented for.
/// O: This is the item that is expected to be the Output of the Operation.
/// `O` might be the same as I or it might be a different type. But `I` must have a relationship with `O`
/// the relationship might be `Many-to-One` or `One-to-One` relationship.
pub trait GetItem<I: Item, O: Item = I>: Sized {
    type Error: ErrorTrait;
    /// This method gets an Item from the database.
    async fn get_item(&self, key: Key<&I::PK, &I::SK>) -> Result<Option<O>, Self::Error>;
}

/// This trait is used to get many items from the database.
/// I: is for the Item on which the trait is implemented for.
/// O: This is the item that is expected to be the Output of the Operation.
/// `O` might be the same as I or it might be a different type. But `I` must have a relationship with `O`
/// the relationship might be `One-to-Many` or `Many-to-Many` relationship.
pub trait GetItems<I: Item, O: Item = I>: Sized {
    type Error: ErrorTrait;
    async fn get_items(&self, key: Key<I::PK, I::SK>) -> Result<Option<Vec<O>>, Self::Error>;
}

/// This trait is used to update an Item
pub trait UpdateItem<I: Item, O: Item = I>: Sized {
    type Error: ErrorTrait;
    /// This method is used to completyley replace an item.
    /// It should also create a new item. if the Item does not exist
    async fn update_item(&self, key: Key<&I::PK, &I::SK>, item: O) -> Result<O, Self::Error>;
    /// This methods is used to update parts of an Item.
    async fn patch_item(&self, key: Key<&I::PK, &I::SK>, map: HashMap<String, Value>) -> Result<O, Self::Error>;
}

/// This trait is used to delete an Item from the database.
pub trait DeleteItem<I: Item>: Sized {
    type Error: ErrorTrait;
    /// This method is used to delete one item from the databse.
    async fn delete_item(&self, key: Key<&I::PK, &I::SK>) -> Result<(), Self::Error>;
    /// This method is used to delete many items from the database.
    async fn delete_items(&self, keys: Vec<Key<&I::PK, &I::SK>>) -> Result<(), Self::Error> {
        for key in keys {
            self.delete_item(key).await?;
        }
        Ok(())
    }
}
