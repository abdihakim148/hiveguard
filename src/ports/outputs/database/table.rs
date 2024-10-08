use std::hash::Hash;
use super::Result;

/// A trait representing a database table.
pub trait Table: Sized {
    /// The type of item stored in the table.
    type Item: Clone + PartialEq;
    /// The type of the identifier for items in the table.
    type Id: Clone + Hash + PartialEq;

    type Map;

    /// The name of the table.
    const NAME: &'static str;

    /// Creates a new instance of the table.
    ///
    /// # Returns
    ///
    /// * `Result<Self>` - Returns a new instance of the table wrapped in a `Result`.
    async fn new() -> Result<Self>;
    /// Creates a new item in the table.
    ///
    /// # Arguments
    ///
    /// * `item` - A reference to the item to be created.
    ///
    /// # Returns
    ///
    /// * `Result<Self::Id>` - Returns the ID of the created item wrapped in a `Result`.
    async fn create(&self, item: &Self::Item) -> Result<Self::Id>;
    /// Reads an item by ID from the table.
    ///
    /// # Arguments
    ///
    /// * `id` - A reference to the ID of the item to be read.
    ///
    /// # Returns
    ///
    /// * `Result<Option<Self::Item>>` - Returns the item if found, otherwise `None`, wrapped in a `Result`.
    async fn read(&self, id: &Self::Id) -> Result<Option<Self::Item>>;


    async fn patch(&self, map: Self::Map) -> Result<Self::Item>;
    /// Updates an existing item in the table.
    ///
    /// # Arguments
    ///
    /// * `item` - A reference to the item to be updated.
    ///
    /// # Returns
    ///
    /// * `Result<Self::Id>` - Returns the ID of the updated item wrapped in a `Result`.
    async fn update(&self, item: &Self::Item) -> Result<()>;
    /// Deletes an item by ID from the table.
    ///
    /// # Arguments
    ///
    /// * `id` - A reference to the ID of the item to be deleted.
    ///
    /// # Returns
    ///
    /// * `Result<Self::Id>` - Returns the ID of the deleted item wrapped in a `Result`.
    async fn delete(&self, id: &Self::Id) -> Result<Self::Item>;
}
