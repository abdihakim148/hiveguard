//! Organisations collection implementation for the memory database
//! 
//! This module provides the implementation for storing and managing organisation records
//! in memory with thread-safe access and index management.

use crate::ports::outputs::database::{Item, CreateItem, GetItem, UpdateItem, DeleteItem};
use crate::domain::types::{Organisation, Id, Key, Value};
use std::collections::HashMap;
use std::sync::RwLock as Lock;
use super::error::Error;

/// Thread-safe, indexed storage for organisation records
/// 
/// # Indexes
/// - Primary index: Organisation ID -> Organisation record
/// - Secondary indexes:
///   * Organisation Name -> Organisation ID
/// 
/// # Concurrency
/// Uses RwLock to ensure safe concurrent read and write operations
#[derive(Debug, Default)]
pub struct Organisations {
    /// Primary storage of organisations, keyed by their unique identifier
    pub organisations: Lock<HashMap<<Organisation as Item>::PK, Organisation>>,
    
    /// Secondary index mapping organisation names to organisation IDs
    /// Enables fast lookups of organisations by their name
    pub names_index: Lock<HashMap<String, <Organisation as Item>::PK>>,
}

impl Organisations {
    /// Updates secondary indexes when an organisation's details change
    /// 
    /// # Arguments
    /// * `pk`: Primary key (Organisation ID) of the organisation being updated
    /// * `sk`: New secondary key (Organisation Name)
    /// 
    /// # Behavior
    /// - Removes old name index
    /// - Adds new name index
    pub fn update_indexes(&self, pk: <Organisation as Item>::PK, sk: <Organisation as Item>::SK) -> Result<(), Error> {
        // Remove old name index if it exists
        if let Some(old_org) = self.organisations.read()?.get(&pk) {
            self.names_index.write()?.remove(&old_org.name);
        }

        // Add new name index
        self.names_index.write()?.insert(sk, pk);
        Ok(())
    }

    /// Finds the primary key for a given secondary key (organisation name)
    pub fn pk(&self, sk: &<Organisation as Item>::SK) -> Result<Option<<Organisation as Item>::PK>, Error> {
        Ok(self.names_index.read()?.get(sk).cloned())
    }

    /// Checks if an organisation with the given name already exists
    pub fn does_not_exist(&self, sk: &<Organisation as Item>::SK) -> Result<(), Error> {
        if self.names_index.read()?.contains_key(sk) {
            return Err(Error::OrganisationWithNameExists);
        }
        Ok(())
    }
}

impl CreateItem<Organisation> for Organisations {
    type Error = Error;
    
    async fn create_item(&self, organisation: Organisation) -> Result<Organisation, Self::Error> {
        // Check if organisation with same name exists
        self.does_not_exist(&organisation.name)?;
        
        // Update indexes
        self.update_indexes(organisation.id, organisation.name.clone())?;
        
        // Store organisation
        self.organisations.write()?.insert(organisation.id, organisation.clone());
        
        Ok(organisation)
    }
}

impl GetItem<Organisation> for Organisations {
    type Error = Error;
    
    async fn get_item(&self, key: Key<&<Organisation as Item>::PK, &<Organisation as Item>::SK>) -> Result<Option<Organisation>, Self::Error> {
        match key {
            Key::Pk(pk) => Ok(self.organisations.read()?.get(pk).cloned()),
            Key::Both((pk, _)) => Ok(self.organisations.read()?.get(pk).cloned()),
            Key::Sk(sk) => {
                if let Some(pk) = self.pk(sk)? {
                    Ok(self.organisations.read()?.get(&pk).cloned())
                } else {
                    Ok(None)
                }
            }
        }
    }
}

impl UpdateItem<Organisation> for Organisations {
    type Error = Error;

    async fn update_item(&self, _: Key<&<Organisation as Item>::PK, &<Organisation as Item>::SK>, organisation: Organisation) -> Result<Organisation, Self::Error> {
        // Update indexes for new organisation
        self.update_indexes(organisation.id, organisation.name.clone())?;
        
        // Store updated organisation
        self.organisations.write()?.insert(organisation.id, organisation.clone());
        Ok(organisation)
    }

    async fn patch_item(&self, key: Key<&<Organisation as Item>::PK, &<Organisation as Item>::SK>, mut map: HashMap<String, Value>) -> Result<Organisation, Self::Error> {
        let id = match key {
            Key::Both((pk, _)) | Key::Pk(pk) => *pk,
            Key::Sk(sk) => match self.pk(sk)? {
                Some(pk) => pk,
                None => return Err(Error::OrganisationNotFound)
            }
        };

        let mut organisations = self.organisations.write()?;
        let organisation = organisations.get_mut(&id).ok_or(Error::OrganisationNotFound)?;
        
        // Update basic fields
        if let Some(value) = map.remove("name") {
            let new_name: String = value.try_into()?;
            self.update_indexes(id, new_name.clone())?;
            organisation.name = new_name;
        }

        if let Some(value) = map.remove("domain") {
            organisation.domain = Some(value.try_into()?);
        }

        if let Some(value) = map.remove("home") {
            organisation.home = Some(value.try_into()?);
        }

        if let Some(value) = map.remove("contacts") {
            organisation.contacts = value.try_into()?;
        }

        Ok(organisation.clone())
    }
}

impl DeleteItem<Organisation> for Organisations {
    type Error = Error;
    
    async fn delete_item(&self, key: Key<&<Organisation as Item>::PK, &<Organisation as Item>::SK>) -> Result<(), Self::Error> {
        let pk = match key {
            Key::Pk(pk) | Key::Both((pk, _)) => *pk,
            Key::Sk(sk) => match self.pk(sk)? {
                Some(pk) => pk,
                None => return Err(Error::OrganisationNotFound)
            }
        };

        // Remove from indexes
        if let Some(organisation) = self.organisations.read()?.get(&pk) {
            self.names_index.write()?.remove(&organisation.name);
        }

        // Remove organisation
        self.organisations.write()?.remove(&pk);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bson::oid::ObjectId;

    /// Helper function to create a test organisation
    fn create_test_organisation() -> Organisation {
        Organisation {
            id: Id(ObjectId::new()),
            name: "Test Organisation".to_string(),
            domain: None,
            home: None,
            contacts: Default::default(),
        }
    }

    #[tokio::test]
    async fn test_create_organisation() {
        let organisations = Organisations::default();
        let organisation = create_test_organisation();
        let result = organisations.create_item(organisation.clone()).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), organisation);
    }

    #[tokio::test]
    async fn test_create_duplicate_organisation() {
        let organisations = Organisations::default();
        let organisation1 = create_test_organisation();
        let mut organisation2 = create_test_organisation();
        organisation2.id = Id(ObjectId::new());
        
        let _ = organisations.create_item(organisation1).await;
        let result = organisations.create_item(organisation2).await;
        assert!(matches!(result, Err(Error::OrganisationWithNameExists)));
    }

    #[tokio::test]
    async fn test_get_organisation_by_id() {
        let organisations = Organisations::default();
        let organisation = create_test_organisation();
        let _ = organisations.create_item(organisation.clone()).await;
        
        let result = organisations.get_item(Key::Pk(&organisation.id)).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Some(organisation));
    }

    #[tokio::test]
    async fn test_get_organisation_by_name() {
        let organisations = Organisations::default();
        let organisation = create_test_organisation();
        let _ = organisations.create_item(organisation.clone()).await;

        let key = Key::Sk(&organisation.name);
        
        let result = organisations.get_item(key).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Some(organisation));
    }

    #[tokio::test]
    async fn test_update_organisation() {
        let organisations = Organisations::default();
        let mut organisation = create_test_organisation();
        let _ = organisations.create_item(organisation.clone()).await;
        
        organisation.name = "Updated Organisation".to_string();
        let result = organisations.update_item(Key::Pk(&organisation.id), organisation.clone()).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), organisation);
    }

    #[tokio::test]
    async fn test_delete_organisation() {
        let organisations = Organisations::default();
        let organisation = create_test_organisation();
        let _ = organisations.create_item(organisation.clone()).await;
        
        let result = organisations.delete_item(Key::Pk(&organisation.id)).await;
        assert!(result.is_ok());
        
        let get_result = organisations.get_item(Key::Pk(&organisation.id)).await;
        assert_eq!(get_result.unwrap(), None);
    }
}
