/// This module defines the `Verifications` struct and its implementation
/// as a table in an in-memory database. It provides methods to create,
/// retrieve, and delete verification records, which are stored in a
/// `HashMap` for quick access.
///
/// The `Verifications` struct implements the `Table` trait, allowing it
/// to be used as a database table with specific operations defined for
/// verification items.
use crate::domain::types::{Either, Key, Value, Verification, Error};
use crate::ports::outputs::database::{Table, Item};
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::sync::RwLock;


#[derive(Debug, Default)]
/// The `Verifications` struct represents a collection of verification
/// records in an in-memory database. It maintains two `HashMap`s:
/// - `secondary`: Maps the secondary key (SK) to the primary key (PK) for
///   quick lookup of verification records by owner.
/// - `primary`: Maps the primary key (PK) to the `Verification`
///   item, allowing retrieval and management of verification data.
pub struct Verifications {
    primary: RwLock<HashMap<<Verification as Item>::PK, Verification>>,
    secondary: RwLock<HashMap<<Verification as Item>::SK, <Verification as Item>::PK>>,
}


impl Table for Verifications {
    type Error = Error;
    type Item = Verification;
    type Map = HashMap<String, Value>;
    const NAME: &'static str = "Verifications";
    
    /// Creates a new instance of the `Verifications` table.
    ///
    /// # Returns
    ///
    /// * `Result<Self, Self::Error>` - A new instance of `Verifications`
    ///   or an error if creation fails.
    async fn new() -> Result<Self, Self::Error> {
        Ok(Default::default())
    }

    /// Inserts a new verification item into the table.
    ///
    /// # Arguments
    ///
    /// * `item` - A reference to the `Verification` item to be inserted.
    ///
    /// # Returns
    ///
    /// * `Result<<Self::Item as Item>::PK, Self::Error>` - The primary key
    ///   of the inserted item or an error if insertion fails.
    async fn create(&self, item: &Self::Item) -> Result<<Self::Item as Item>::PK, Self::Error> {
        let id = item.id;
        let (mut secondary, mut primary) = (self.secondary.write()?, self.primary.write()?);
        let res = (secondary.insert(item.owner_id, item.id), primary.insert(item.id, item.clone()));
        Ok(id)
    }

    /// Retrieves a verification item by its primary or secondary key.
    ///
    /// # Arguments
    ///
    /// * `key` - An `Either` type representing the primary key (PK) or
    ///   secondary key (SK) used for lookup.
    ///
    /// # Returns
    ///
    /// * `Result<Option<Self::Item>, Self::Error>` - The found item or
    ///   `None` if no item matches the key, or an error if retrieval fails.
    async fn get(&self, key: Either<&<Self::Item as Item>::PK, &<Self::Item as Item>::SK>) -> Result<Option<Self::Item>, Self::Error> {
        let secondary = self.secondary.read()?;
        let pk = match key {
            Either::Left(pk) => pk,
            Either::Right(sk) => {
                match secondary.get(sk) {
                    Some(pk) => pk,
                    None => return Ok(None)
                }
            }
        };
        let primary = self.primary.read()?;
        Ok(primary.get(pk).cloned())
    }

    async fn get_many(&self, key: Key<&<Self::Item as Item>::PK, &<Self::Item as Item>::SK>) -> Result<Option<Vec<Self::Item>>, Self::Error> {
        unimplemented!()
    }

    async fn patch(&self, id: &<Self::Item as Item>::PK, map: Self::Map) -> Result<Self::Item, Self::Error> {
        unimplemented!()
    }

    async fn update(&self, item: &Self::Item) -> Result<(), Self::Error> {
        unimplemented!()
    }

    /// Deletes a verification item by its primary key.
    ///
    /// # Arguments
    ///
    /// * `id` - A reference to the primary key of the item to be deleted.
    ///
    /// # Returns
    ///
    /// * `Result<(), Self::Error>` - An empty result indicating success
    ///   or an error if deletion fails.
    async fn delete(&self, id: &<Self::Item as Item>::PK) -> Result<(), Self::Error> {
        let (mut primary, mut secondary) = (self.primary.write()?, self.secondary.write()?);
        let verification = match primary.remove(id) {
            Some(verification) => verification,
            None => return  Ok(())
        };
        secondary.remove(&verification.owner_id);
        Ok(())
    }
}



#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::types::{Id, Verification};
    use chrono::Utc;

    #[tokio::test]
    async fn test_create_and_get_verification() {
        let primary = Verifications::new().await.unwrap();
        let verification = Verification::default();

        // Test creation
        let pk = primary.create(&verification).await.unwrap();
        assert_eq!(pk, verification.id);

        // Test retrieval by primary key
        let retrieved = primary.get(Either::Left(&verification.id)).await.unwrap();
        assert_eq!(retrieved, Some(verification.clone()));

        // Test retrieval by secondary key
        let retrieved_by_sk = primary.get(Either::Right(&verification.owner_id)).await.unwrap();
        assert_eq!(retrieved_by_sk, Some(verification));
    }

    #[tokio::test]
    async fn test_delete_verification() {
        let primary = Verifications::new().await.unwrap();
        let verification = Verification::default();

        // Create and then delete the verification
        primary.create(&verification).await.unwrap();
        primary.delete(&verification.id).await.unwrap();

        // Ensure it cannot be retrieved
        let retrieved = primary.get(Either::Left(&verification.id)).await.unwrap();
        assert!(retrieved.is_none());
    }
}