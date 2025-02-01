use crate::domain::types::{ErrorTrait, Error};
use crate::domain::types::{Either, Key};
use super::Item;

/// A trait representing a database table.
pub trait Table: Sized {

    type Map;

    type Error: ErrorTrait + Into<Error>;

    type Item: Clone + PartialEq + Item;

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
    async fn create(&self, item: &Self::Item) -> Result<<Self::Item as Item>::PK, Self::Error>;
    /// Reads an item by ID from the table.
    ///
    /// # Arguments
    ///
    /// * `id` - A reference to the ID of the item to be read.
    ///
    /// # Returns
    ///
    /// * `Result<Option<Item>>` - Returns the item if found, otherwise `None`, wrapped in a `Result`.
    async fn get(&self, key: Either<&<Self::Item as Item>::PK, &<Self::Item as Item>::SK>) -> Result<Option<Self::Item>, Self::Error>;


    /// Reads multiple items by a composite key from the table.
    ///
    /// # Arguments
    ///
    /// * `key` - A composite key consisting of primary and secondary keys.
    ///
    /// # Returns
    ///
    /// * `Result<Option<Vec<impl Iterator<Item = Self::Item>>>>` - Returns an iterator over the items if found, otherwise `None`, wrapped in a `Result`.
    async fn get_many(&self, key: Key<&<Self::Item as Item>::PK, &<Self::Item as Item>::SK>) -> Result<Option<Vec<Self::Item>>, Self::Error>;


    /// Partially updates an existing item in the table.
    ///
    /// # Arguments
    ///
    /// * `id` - A reference to the ID of the item to be patched.
    /// * `map` - A map of fields and values to update.
    ///
    /// # Returns
    ///
    /// * `Result<Item>` - Returns the updated item wrapped in a `Result`.
    async fn patch(&self, id: &<Self::Item as Item>::PK, map: Self::Map) -> Result<Self::Item, Self::Error>;
    /// Replaces an existing item in the table with a new one.
    ///
    /// # Arguments
    ///
    /// * `item` - A reference to the item to be updated.
    ///
    /// # Returns
    ///
    /// * `Result<()>` - Returns an empty result on success.
    async fn update(&self, item: &Self::Item) -> Result<(), Self::Error>;
    /// Deletes an item by ID from the table.
    ///
    /// # Arguments
    ///
    /// * `id` - A reference to the ID of the item to be deleted.
    ///
    /// # Returns
    ///
    /// * `Result<()>` - Returns an empty result on success.
    async fn delete(&self, id: &<Self::Item as Item>::PK) -> Result<(), Self::Error>;
}
