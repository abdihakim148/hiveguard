use crate::ports::{ErrorTrait, Error};
use crate::domain::types::Either;
use super::Item as ItemTrait;
use std::hash::Hash;

/// A trait representing a database table.
pub trait Table<Item: Clone + PartialEq + ItemTrait>: Sized {

    type Map;

    type Error: ErrorTrait + Into<Error>;

    /// The name of the table.
    const NAME: &'static str;

    /// Creates a new instance of the table.
    ///
    /// # Returns
    ///
    /// * `Result<Self>` - Returns a new instance of the table wrapped in a `Result`.
    async fn new() -> Result<Self, Self::Error>;
    /// Creates a new item in the table.
    ///
    /// # Arguments
    ///
    /// * `item` - A reference to the item to be created.
    ///
    /// # Returns
    ///
    /// * `Result<Item::PK>` - Returns the ID of the created item wrapped in a `Result`.
    async fn create(&self, item: &Item) -> Result<Item::PK, Self::Error>;
    /// Reads an item by ID from the table.
    ///
    /// # Arguments
    ///
    /// * `id` - A reference to the ID of the item to be read.
    ///
    /// # Returns
    ///
    /// * `Result<Option<Item>>` - Returns the item if found, otherwise `None`, wrapped in a `Result`.
    async fn get(&self, key: Either<&Item::PK, &Item::SK>) -> Result<Option<Item>, Self::Error>;


    async fn patch(&self, id: &Item::PK, map: Self::Map) -> Result<Item, Self::Error>;
    /// Updates an existing item in the table.
    ///
    /// # Arguments
    ///
    /// * `item` - A reference to the item to be updated.
    ///
    /// # Returns
    ///
    /// * `Result<Item::PK>` - Returns the ID of the updated item wrapped in a `Result`.
    async fn update(&self, item: &Item) -> Result<(), Self::Error>;
    /// Deletes an item by ID from the table.
    ///
    /// # Arguments
    ///
    /// * `id` - A reference to the ID of the item to be deleted.
    ///
    /// # Returns
    ///
    /// * `Result<Item::PK>` - Returns the ID of the deleted item wrapped in a `Result`.
    async fn delete(&self, id: &Item::PK) -> Result<(), Self::Error>;
}
