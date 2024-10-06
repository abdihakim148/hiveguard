use crate::ports::output::database::{Table as DbTable, Result};
use bson::oid::ObjectId;

/// A trait providing CRUD operations for types with a corresponding database table.
pub trait Crud: Sized {
    type Table: DbTable<Item = Self>;

    /// Creates a new item in the table.
    async fn create(table: &Self::Table, item: &Self) -> Result<ObjectId> {
        table.create(item).await
    }

    /// Reads an item by ID from the table.
    async fn read(table: &Self::Table, id: &ObjectId) -> Result<Option<Self>> {
        table.read(id).await
    }

    /// Updates an existing item in the table.
    async fn update(table: &Self::Table, item: &Self) -> Result<ObjectId> {
        table.update(item).await
    }

    /// Deletes an item by ID from the table.
    async fn delete(table: &Self::Table, id: &ObjectId) -> Result<ObjectId> {
        table.delete(id).await
    }

    /// Checks if an item with the given email exists in the table.
    async fn exists(table: &Self::Table, email: &str) -> Result<bool> {
        table.exists(email).await
    }
}
