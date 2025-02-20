//! Organisations collection implementation for the memory database
//! 
//! This module provides the implementation for storing and managing organisation records
//! in memory with thread-safe access and index management.

use crate::ports::outputs::database::{Item, CreateItem, GetItem, UpdateItem, DeleteItem, Map};
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
    
    async fn get_item(&self, key: Key<&<Organisation as Item>::PK, &<Organisation as Item>::SK>) -> Result<Organisation, Self::Error> {
        let option = match key {
            Key::Pk(pk) => self.organisations.read()?.get(pk).cloned(),
            Key::Both((pk, _)) => self.organisations.read()?.get(pk).cloned(),
            Key::Sk(sk) => {
                if let Some(pk) = self.pk(sk)? {
                    self.organisations.read()?.get(&pk).cloned()
                } else {
                    None
                }
            }
        };

        if let Some(service) = option {
            return Ok(service)
        }
        Err(Error::ServiceNotFound)
    }
}

impl UpdateItem<Organisation> for Organisations {
    type Error = Error;
    type Update = Map;

    async fn update_item(&self, _: Key<&<Organisation as Item>::PK, &<Organisation as Item>::SK>, organisation: Organisation) -> Result<Organisation, Self::Error> {
        // Update indexes for new organisation
        self.update_indexes(organisation.id, organisation.name.clone())?;
        
        // Store updated organisation
        self.organisations.write()?.insert(organisation.id, organisation.clone());
        Ok(organisation)
    }

    /// Partially update an organisation's fields
    /// 
    /// # Arguments
    /// * `key`: The key to identify the organisation to update
    /// * `map`: A map of fields to update
    /// 
    /// # Returns
    /// The updated organisation or an error if the update fails
    /// 
    /// # Behavior
    /// - Allows updating name, domain, home, and contacts
    /// - Updates secondary indexes if name changes
    async fn patch_item(&self, key: Key<&<Organisation as Item>::PK, &<Organisation as Item>::SK>, map: Map) -> Result<Organisation, Self::Error> {
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
        if let Some(value) = map.get("name") {
            let new_name: String = value.clone().try_into()?;
            self.update_indexes(id, new_name.clone())?;
            organisation.name = new_name;
        }

        if let Some(value) = map.get("domain") {
            organisation.domain = Some(value.clone().try_into()?);
        }

        if let Some(value) = map.get("home") {
            organisation.home = Some(value.clone().try_into()?);
        }

        if let Some(value) = map.get("contacts") {
            organisation.contacts = value.clone().try_into()?;
        }

        Ok(organisation.clone())
    }

    /// Delete specific fields from an organisation
    /// 
    /// # Arguments
    /// * `key`: The key to identify the organisation to update
    /// * `fields`: List of field names to delete
    /// 
    /// # Returns
    /// The updated organisation or an error if the deletion fails
    /// 
    /// # Behavior
    /// - Organisations do not support deleting individual fields
    async fn delete_fields(&self, _key: Key<&<Organisation as Item>::PK, &<Organisation as Item>::SK>, _fields: &[String]) -> Result<Organisation, Self::Error> {
        Err(Error::UnsupportedOperation)
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
        assert_eq!(result.unwrap(), organisation);
    }

    #[tokio::test]
    async fn test_get_organisation_by_name() {
        let organisations = Organisations::default();
        let organisation = create_test_organisation();
        let _ = organisations.create_item(organisation.clone()).await;

        let key = Key::Sk(&organisation.name);
        
        let result = organisations.get_item(key).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), organisation);
    }

    #[tokio::test]
    async fn test_patch_organisation_name() {
        let organisations = Organisations::default();
        let organisation = create_test_organisation();
        let _ = organisations.create_item(organisation.clone()).await;
        
        let patch_map = HashMap::from([
            ("name".to_string(), Value::String("Updated Organisation".to_string()))
        ]);

        let result = organisations.patch_item(Key::Pk(&organisation.id), patch_map).await;
        assert!(result.is_ok(), "Patching organisation name should succeed");
        
        let updated_org = result.unwrap();
        assert_eq!(updated_org.name, "Updated Organisation", "Organisation name should be updated");
    }

    #[tokio::test]
    async fn test_patch_organisation_domain() {
        let organisations = Organisations::default();
        let organisation = create_test_organisation();
        let _ = organisations.create_item(organisation.clone()).await;
        
        let patch_map = HashMap::from([
            ("domain".to_string(), Value::String("example.com".to_string()))
        ]);

        let result = organisations.patch_item(Key::Pk(&organisation.id), patch_map).await;
        assert!(result.is_ok(), "Patching organisation domain should succeed");
        
        let updated_org = result.unwrap();
        assert_eq!(updated_org.domain, Some("example.com".to_string()), "Organisation domain should be updated");
    }

    #[tokio::test]
    async fn test_delete_organisation_fields_unsupported() {
        let organisations = Organisations::default();
        let organisation = create_test_organisation();
        let _ = organisations.create_item(organisation.clone()).await;
        
        let result = organisations.delete_fields(Key::Pk(&organisation.id), &["name".to_string()]).await;
        assert!(matches!(result, Err(Error::UnsupportedOperation)), 
                "Deleting fields should return UnsupportedOperation");
    }
}
