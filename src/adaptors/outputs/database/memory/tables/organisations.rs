/// This module defines the `Organisations` table which implements the `Table` trait.
/// It provides asynchronous methods to create, retrieve, update, and delete organisations
/// in a memory-based database using primary and secondary indices.

use crate::adaptors::outputs::database::memory::tables::organisations;
use crate::domain::types::{Either, Key, Value, Organisation, Error};
use crate::ports::outputs::database::{Table, Item};
use std::collections::HashMap;
use std::sync::RwLock;

#[derive(Default, Debug)]
/// The `Organisations` struct represents a table of organisations with primary and secondary indices.
/// The primary index is a map from the organisation's primary key (PK) to the organisation itself.
/// The secondary index is a map from the organisation's secondary key (SK) to the primary key (PK).
pub struct Organisations {
    primary: RwLock<HashMap<<Organisation as Item>::PK, Organisation>>,
    secondary: RwLock<HashMap<<Organisation as Item>::SK, <Organisation as Item>::PK>>
}


impl Table for Organisations {
    type Error = Error;
    type Item = Organisation;
    type Map = HashMap<String, Value>;
    const NAME: &'static str = "Organisations";
    
    /// Creates a new instance of the `Organisations` table.
    ///
    /// # Returns
    ///
    /// * `Result<Self, Self::Error>` - A new instance of `Organisations` or an error.
    async fn new() -> Result<Self, Self::Error> {
        Ok(Default::default())
    }

    /// Inserts a new organisation into the table.
    ///
    /// # Arguments
    ///
    /// * `item` - A reference to the organisation to be inserted.
    ///
    /// # Returns
    ///
    /// * `Result<<Self::Item as Item>::PK, Self::Error>` - The primary key of the inserted organisation or an error.
    ///
    /// In this context, PK is the `id` and SK is the `name`.
    async fn create(&self, item: &Self::Item) -> Result<<Self::Item as Item>::PK, Self::Error> {
        // Check for conflicts in both primary and secondary indices.
        let (name, id) = (item.name.clone(), item.id);
        let (mut secondary, mut primary) = (self.secondary.write()?, self.primary.write()?);
        if let Some(_) = secondary.get(&name) {
            return Err(Error::Conflict(Self::Item::NAME))
        }
        if let Some(_) = primary.get(&id) {
            return Err(Error::Conflict(Self::Item::NAME))
        }
        secondary.insert(name, id);
        primary.insert(id, item.clone());
        Ok(id)
    }

    /// Retrieves an organisation from the table using a key.
    ///
    /// # Arguments
    ///
    /// * `key` - A key that can be either a primary key or a combination of primary and secondary keys.
    ///
    /// # Returns
    ///
    /// * `Result<Option<Self::Item>, Self::Error>` - The organisation if found, or None if not found.
    ///
    /// The SK is used here to resolve the primary key if only the SK is provided.
    async fn get(&self, key: Key<&<Self::Item as Item>::PK, &<Self::Item as Item>::SK>) -> Result<Option<Self::Item>, Self::Error> {
        let pk = match key {
            Key::Pk(pk) => *pk,
            Key::Sk(sk) => {
                match self.secondary.read()?.get(sk) {
                    None => return Ok(None),
                    Some(pk) => *pk
                }
            },
            Key::Both((pk, _)) => *pk
        };
        Ok(self.primary.read()?.get(&pk).cloned())
    }

    /// Retrieves multiple organisations associated with a primary or secondary key.
    ///
    /// # Arguments
    ///
    /// * `key` - An `Either` type that can be a primary key or a secondary key.
    ///
    /// # Returns
    ///
    /// * `Result<Option<Vec<Self::Item>>, Self::Error>` - A vector of organisations if found, or None if not found.
    ///
    /// This method is currently unimplemented.
    async fn get_many(&self, key: Either<&<Self::Item as Item>::PK, &<Self::Item as Item>::SK>) -> Result<Option<Vec<Self::Item>>, Self::Error> {
        unimplemented!()
    }

    /// Updates specific fields of an organisation identified by its primary key.
    ///
    /// # Arguments
    ///
    /// * `id` - The primary key of the organisation to be updated.
    /// * `map` - A map containing the fields to be updated.
    ///
    /// # Returns
    ///
    /// * `Result<Self::Item, Self::Error>` - The updated organisation or an error.
    async fn patch(&self, id: &<Self::Item as Item>::PK, mut map: Self::Map) -> Result<Self::Item, Self::Error> {
        // Retrieve the existing organisation and update the specified fields.
        let key = Key::Pk(id);
        if let Some(organisation) = self.get(key).await? {
            let id = *id;
            let name = match map.remove("name") {
                Some(value) => value.try_into()?,
                None => organisation.name.clone()
            };
            let owners = match map.remove("owners") {
                Some(value) => value.try_into()?,
                None => organisation.owners.clone()
            };
            let domain = match map.remove("domain") {
                Some(value) => Some(value.try_into()?),
                None => organisation.domain.clone()
            };
            let home = match map.remove("home") {
                Some(value) => Some(value.try_into()?),
                None => organisation.home.clone()
            };
            let contacts = match map.remove("contacts") {
                Some(value) => value.try_into()?,
                None => organisation.contacts.clone()
            };
            let item = Organisation{id, name, owners, domain, home, contacts};
            self.update(&item).await?;
            return Ok(item);
        }
        Err(Error::NotFound(Self::Item::NAME))
    }

    /// Updates an organisation in the table.
    ///
    /// # Arguments
    ///
    /// * `item` - A reference to the organisation to be updated.
    ///
    /// # Returns
    ///
    /// * `Result<(), Self::Error>` - An empty result or an error.
    async fn update(&self, item: &Self::Item) -> Result<(), Self::Error> {
        // Ensure the organisation's name is unique in the secondary index.
        let mut primary = self.primary.write()?;
        let mut secondary = self.secondary.write()?;

        if let Some(id) = secondary.get(&item.name) {
            if id != &item.id {
                return Err(Error::Conflict(Self::Item::NAME));
            }
        }

        if let Some(existing_organisation) = primary.get_mut(&item.id) {
            secondary.remove(&existing_organisation.name);
            *existing_organisation = item.clone();
            secondary.insert(item.name.clone(), item.id.clone());
        }else {
            self.create(item).await?;
        }

        Ok(())
    }

    /// Deletes an organisation from the table using its primary key.
    ///
    /// # Arguments
    ///
    /// * `id` - The primary key of the organisation to be deleted.
    ///
    /// # Returns
    ///
    /// * `Result<(), Self::Error>` - An empty result or an error.
    async fn delete(&self, id: &<Self::Item as Item>::PK) -> Result<(), Self::Error> {
        // Remove the organisation from both primary and secondary indices.
        let (mut primary, mut secondary) = (self.primary.write()?, self.secondary.write()?);
        let item = match primary.remove(id) {
            Some(item) => item,
            None => return Ok(())
        };
        secondary.remove(&item.name);
        Ok(())
    }
}



#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::types::{Id, Organisation};
    use std::collections::HashMap;

    #[tokio::test]
    async fn test_create_organisation() {
        let organisations = Organisations::new().await.unwrap();
        let organisation = Organisation {
            id: Id::default(),
            name: "Test Organisation".to_string(),
            owners: vec![Id::default()],
            domain: Some("example.com".to_string()),
            home: Some("http://example.com".to_string()),
            contacts: Default::default(),
        };

        let result = organisations.create(&organisation).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_organisation() {
        let organisations = Organisations::new().await.unwrap();
        let organisation = Organisation {
            id: Id::default(),
            name: "Test Organisation".to_string(),
            owners: vec![Id::default()],
            domain: Some("example.com".to_string()),
            home: Some("http://example.com".to_string()),
            contacts: Default::default(),
        };

        organisations.create(&organisation).await.unwrap();
        let result = organisations.get(Key::Pk(&organisation.id)).await.unwrap();
        assert_eq!(result, Some(organisation));
    }

    #[tokio::test]
    async fn test_update_organisation() {
        let organisations = Organisations::new().await.unwrap();
        let mut organisation = Organisation {
            id: Id::default(),
            name: "Test Organisation".to_string(),
            owners: vec![Id::default()],
            domain: Some("example.com".to_string()),
            home: Some("http://example.com".to_string()),
            contacts: Default::default(),
        };

        organisations.create(&organisation).await.unwrap();
        organisation.name = "Updated Organisation".to_string();
        organisations.update(&organisation).await.unwrap();
        let result = organisations.get(Key::Pk(&organisation.id)).await.unwrap();
        assert_eq!(result.unwrap().name, "Updated Organisation");
    }

    #[tokio::test]
    async fn test_delete_organisation() {
        let organisations = Organisations::new().await.unwrap();
        let organisation = Organisation {
            id: Id::default(),
            name: "Test Organisation".to_string(),
            owners: vec![Id::default()],
            domain: Some("example.com".to_string()),
            home: Some("http://example.com".to_string()),
            contacts: Default::default(),
        };

        organisations.create(&organisation).await.unwrap();
        organisations.delete(&organisation.id).await.unwrap();
        let result = organisations.get(Key::Pk(&organisation.id)).await.unwrap();
        assert!(result.is_none());
    }
}