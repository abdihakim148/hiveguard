use std::collections::{HashMap, HashSet};
use crate::domain::types::{Key, Value};
use crate::ports::ErrorTrait;
use super::Item;


pub type Map = HashMap<String, Value>;

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
    async fn get_item(&self, key: Key<&I::PK, &I::SK>) -> Result<O, Self::Error>;
}

/// This trait is used to get many items from the database.
/// I: is for the Item on which the trait is implemented for.
/// O: This is the item that is expected to be the Output of the Operation.
/// `O` might be the same as I or it might be a different type. But `I` must have a relationship with `O`
/// the relationship might be `One-to-Many` or `Many-to-Many` relationship.
pub trait GetItems<I: Item, O: Item = I>: Sized {
    type Error: ErrorTrait;
    type Filter;
    /// Retrieve multiple items based on a key and filter
    /// 
    /// # Arguments
    /// * `key`: The primary or secondary key to filter by
    /// * `filter`: Additional filtering criteria
    /// 
    /// # Returns
    /// * `Some(Vec<O>)` if items are found
    /// * `None` if no items match the criteria
    async fn get_items(&self, key: Key<&I::PK, &I::SK>, filter: Self::Filter) -> Result<Vec<O>, Self::Error>;
}

/// This trait is used to update an Item
pub trait UpdateItem<I: Item, O: Item = I>: Sized {
    type Error: ErrorTrait;
    type Update;

    /// This method is used to completely replace an item.
    /// It should also create a new item if the Item does not exist
    async fn update_item(&self, key: Key<&I::PK, &I::SK>, item: O) -> Result<O, Self::Error>;
    
    /// This method is used to add or replace fields of an Item
    /// 
    /// # Arguments
    /// * `key`: The key to identify the item to update
    /// * `update`: Specification of fields to add or replace
    /// 
    /// # Returns
    /// The updated item or an error if the update fails
    async fn patch_item(&self, key: Key<&I::PK, &I::SK>, update: Self::Update) -> Result<O, Self::Error>;

    /// This method is used to delete specific fields from an Item
    /// 
    /// # Arguments
    /// * `key`: The key to identify the item to update
    /// * `fields`: List of field names to delete
    /// 
    /// # Returns
    /// The updated item or an error if the deletion fails
    async fn delete_fields(&self, key: Key<&I::PK, &I::SK>, fields: HashSet<String>) -> Result<O, Self::Error>;
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
