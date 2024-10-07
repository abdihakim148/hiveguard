#![allow(unused)]
use crate::ports::outputs::database::Table;

/// A type alias for results returned by CRUD operations, using the custom `Error` type.
type Result<T> = std::result::Result<T, crate::domain::types::Error>;

/// A trait providing CRUD operations for types with a corresponding database table.
pub trait Crud: Sized {
    type Id;

    /// Creates a new item in the table.
    async fn create<T: Table<Item = Self, Id = Self::Id>>(&self, table: &T) -> Result<Self::Id> {
        table.create(self).await
    }

    /// Reads an item by ID from the table.
    async fn read<T: Table<Item = Self, Id = Self::Id>>(table: &T, id: &Self::Id) -> Result<Option<Self>> {
        table.read(id).await
    }

    /// Updates an existing item in the table.
    async fn update<T: Table<Item = Self, Id = Self::Id>>(&self, table: &T) -> Result<Self::Id> {
        table.update(self).await
    }

    /// Deletes an item by ID from the table.
    async fn delete<T: Table<Item = Self, Id = Self::Id>>(table: &T, id: &Self::Id) -> Result<Self::Id> {
        table.delete(id).await
    }
}
