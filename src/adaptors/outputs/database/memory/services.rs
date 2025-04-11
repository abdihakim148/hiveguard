//! Services collection implementation for the memory database
//!
//! This module provides a thread-safe, in-memory implementation for storing and managing service records.
//! 
//! # Key Features
//! - Thread-safe storage using RwLock
//! - Unique service names per owner
//! - Efficient indexing for quick lookups
//! 
//! # Indexes
//! - Primary index: Service ID -> Service record
//! - Secondary index: Owner ID -> (Service Name -> Service ID)
//! 
//! # Concurrency
//! Uses RwLock to ensure safe concurrent read and write operations

use crate::ports::outputs::database::{CreateItem, DeleteItem, GetItem, Item, UpdateItem, Map};
use crate::domain::types::{Id, Key, Service, Value};
use std::collections::{HashMap, HashSet};
use std::sync::RwLock as Lock;
use super::error::Error;
use chrono::Duration;

/// Thread-safe, indexed storage for service records
///
/// # Indexes
/// - Primary index: Service ID -> Service record
/// - Secondary indexes:
///   * Owner ID -> Service Names
///
/// # Concurrency
/// Uses RwLock to ensure safe concurrent read and write operations
#[derive(Debug, Default)]
pub struct Services {
    /// Primary storage of services, keyed by service ID
    pub services: Lock<HashMap<<Service as Item>::PK, Service>>,

    /// Secondary index mapping owner IDs to service names and their corresponding IDs
    /// This ensures name uniqueness within an owner's context
    pub owner_index: Lock<HashMap<Id, HashMap<String, Id>>>,
}

impl Services {
    /// Checks if a service with the given name already exists for this owner
    ///
    /// # Arguments
    /// * `owner_id` - The ID of the service owner
    /// * `name` - The name of the service to check
    ///
    /// # Returns
    /// * `Ok(())` if no service with this name exists for the owner
    /// * `Err(ServiceAlreadyExists)` if a service with this name already exists
    pub fn does_not_exist(&self, owner_id: &Id, name: &str) -> Result<(), Error> {
        let owner_index = self.owner_index.read()?;
        match owner_index.get(owner_id) {
            Some(owner_services) if owner_services.contains_key(name) => 
                Err(Error::ServiceAlreadyExists),
            _ => Ok(())
        }
    }

    /// Find the service ID for a given owner and name
    ///
    /// # Arguments
    /// * `owner_id` - The ID of the service owner
    /// * `name` - The name of the service
    ///
    /// # Returns
    /// * `Ok(Some(service_id))` if a service is found
    /// * `Ok(None)` if no service is found
    pub fn pk(&self, owner_id: &Id, name: &str) -> Result<Option<Id>, Error> {
        Ok(self.owner_index.read()
            .map(|index| 
                index.get(owner_id)
                    .and_then(|services| services.get(name).cloned())
            )?)
    }
}

impl CreateItem<Service> for Services {
    type Error = Error;

    async fn create_item(&self, service: Service) -> Result<Service, Self::Error> {
        // Check if service with same name exists for this owner
        self.does_not_exist(&service.owner_id, &service.name)?;

        // Update indexes
        let mut owner_index = self.owner_index.write()?;
        owner_index
            .entry(service.owner_id.clone())
            .or_default()
            .insert(service.name.clone(), service.id.clone());

        // Store service
        self.services
            .write()?
            .insert(service.id.clone(), service.clone());

        Ok(service)
    }
}

impl GetItem<Service> for Services {
    type Error = Error;

    async fn get_item(
        &self,
        key: Key<&<Service as Item>::PK, &<Service as Item>::SK>,
    ) -> Result<Option<Service>, Self::Error> {
        let option = match key {
            Key::Pk(pk) | Key::Both((pk, _)) => self.services.read().map(|services| services.get(pk).cloned())?,
            Key::Sk(sk) => { // Look up by name across all owners
                let owner_index = self.owner_index.read()?;
                owner_index
                    .iter()
                    .find_map(|(_, owner_services)| 
                        owner_services.get(sk)
                            .and_then(|service_id| self.services.read().map(|services| services.get(service_id).cloned()).ok())
                    )
                    .flatten() // Flatten Option<Option<Service>> to Option<Service>
            }
        };

        Ok(option)
    }
}

impl UpdateItem<Service> for Services {
    type Error = Error;
    type Update = Map;

    async fn update_item(
        &self,
        _: Key<&<Service as Item>::PK, &<Service as Item>::SK>,
        service: Service,
    ) -> Result<Service, Self::Error> {
        // Check if another service with the same name exists for this owner
        self.does_not_exist(&service.owner_id, &service.name)?;

        // Update owner index
        let mut owner_index = self.owner_index.write()?;
        owner_index
            .entry(service.owner_id.clone())
            .or_default()
            .insert(service.name.clone(), service.id.clone());

        // Store updated service
        let mut services = self.services.write()?;
        services.insert(service.id.clone(), service.clone());

        Ok(service)
    }

    async fn patch_item(
        &self,
        key: Key<&<Service as Item>::PK, &<Service as Item>::SK>,
        map: Map,
    ) -> Result<Service, Self::Error> {
        // First, retrieve the existing service
        let mut service = self.get_item(key.clone()).await?.ok_or(Error::UnsupportedOperation)?; // Cannot patch non-existent service
        
        // Update basic fields
        if let Some(value) = map.get("new_name") {
            let new_name: String = value.clone().try_into()?;

            // Validate old_name matches the current service name
            // Validate old_name matches the current service name if provided
            if let Some(old_name_value) = map.get("old_name") {
                let old_name: String = old_name_value.clone().try_into()?;
                if old_name != service.name {
                    // If old_name is provided but doesn't match, treat as error
                    return Err(Error::UnsupportedOperation); 
                }
            }
            
            // Check if the new name already exists for the owner before changing
            self.does_not_exist(&service.owner_id, &new_name)?;
            
            // Remove old name from index before updating service name
            if let Some(owner_services) = self.owner_index.write()?.get_mut(&service.owner_id) {
                owner_services.remove(&service.name);
            }

            service.name = new_name;
        }

        // Update other fields
        if let Some(value) = map.get("client_secret") {
            service.client_secret = value.clone().try_into()?;
        }
        if let Some(value) = map.get("redirect_uris") {
            service.redirect_uris = value.clone().try_into()?;
        }
        if let Some(value) = map.get("scopes") {
            service.scopes = value.clone().try_into()?;
        }
        if let Some(value) = map.get("grant_types") {
            service.grant_types = value.clone().try_into()?;
        }
        if let Some(value) = map.get("token_expiry") {
            let (seconds,): (i64,) = value.clone().try_into()?;
            service.token_expiry = Some(Duration::seconds(seconds));
        }

        // Use update_item to handle indexes and storage
        self.update_item(key, service).await
    }

    async fn delete_fields(&self, key: Key<&<Service as Item>::PK, &<Service as Item>::SK>, fields: HashSet<String>) -> Result<Service, Self::Error> {
        // Services do not support deleting fields
        Err(Error::UnsupportedOperation)
    }
}

impl DeleteItem<Service> for Services {
    type Error = Error;

    async fn delete_item(
        &self,
        key: Key<&<Service as Item>::PK, &<Service as Item>::SK>,
    ) -> Result<(), Self::Error> {
        // Retrieve the service first to get its details for index removal
        let service = match self.get_item(key).await? {
            Some(service) => service,
            None => return Ok(()) // Service not found, deletion is idempotent
        };

        // Remove from primary services map
        if self.services.write()?.remove(&service.id).is_none() {
             return Ok(()); // Already removed by another thread, idempotent
        }

        // Remove from owner index
        let mut owner_index = self.owner_index.write()?;
        if let Some(owner_services) = owner_index.get_mut(&service.owner_id) {
            owner_services.remove(&service.name);

            // If no more services for this owner, remove the owner entry
            if owner_services.is_empty() {
                owner_index.remove(&service.owner_id);
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::types::{GrantType, Permission, Scope};
    use bson::oid::ObjectId;
    use chrono::Duration;

    /// Helper function to create a test service
    fn create_test_service() -> Service {
        Service {
            id: Id(ObjectId::new()),
            owner_id: Id(ObjectId::new()),
            name: "Test Service".to_string(),
            client_secret: "secret".to_string(),
            redirect_uris: vec!["http://localhost".to_string()],
            scopes: vec![Scope {
                id: Id(ObjectId::new()),
                name: "test_scope".to_string(),
                permission: Permission::Read,
            }],
            grant_types: vec![GrantType::AuthorizationCode],
            token_expiry: Some(Duration::hours(1)),
        }
    }

    #[tokio::test]
    async fn test_create_service() {
        let services = Services::default();
        let service = create_test_service();
        let result = services.create_item(service.clone()).await;
        assert!(result.is_ok(), "Service creation should succeed");
        assert_eq!(result.unwrap(), service, "Created service should match input");
    }

    #[tokio::test]
    async fn test_create_duplicate_service_name_same_owner() {
        let services = Services::default();
        let service1 = create_test_service();
        let service2 = service1.clone();

        let _ = services.create_item(service1).await;
        let result = services.create_item(service2).await;
        assert!(matches!(result, Err(Error::ServiceAlreadyExists)), 
                "Creating a service with duplicate name for same owner should fail");
    }

    #[tokio::test]
    async fn test_create_service_with_same_name_different_owner() {
        let services = Services::default();
        let mut service1 = create_test_service();
        let mut service2 = create_test_service();
        service2.owner_id = Id(ObjectId::new());

        let _ = services.create_item(service1.clone()).await;
        let result = services.create_item(service2.clone()).await;
        assert!(result.is_ok(), "Services with same name but different owners should be allowed");
    }

    #[tokio::test]
    async fn test_patch_service_name() {
        let services = Services::default();
        let service = create_test_service();
        let _ = services.create_item(service.clone()).await;
        let original_name = service.name.clone();

        let patch_map = HashMap::from([
            ("new_name".to_string(), Value::String("Updated Service".to_string())),
            ("old_name".to_string(), Value::String(original_name.clone())) // Provide correct old name
        ]);

        let result = services.patch_item(Key::Pk(&service.id), patch_map).await;
        assert!(result.is_ok(), "Patching service name should succeed: {:?}", result.err());
        let updated_service = result.unwrap();
        assert_eq!(updated_service.name, "Updated Service", "Service name should be updated");

        // Verify old name is gone from index
        let old_pk = services.pk(&service.owner_id, &original_name).unwrap();
        assert!(old_pk.is_none(), "Old service name should not exist in index");

        // Verify new name exists in index
        let new_pk = services.pk(&service.owner_id, "Updated Service").unwrap();
        assert!(new_pk.is_some(), "New service name should exist in index");
        assert_eq!(new_pk.unwrap(), service.id);
    }

    #[tokio::test]
    async fn test_delete_service_unsupported_fields() {
        let services = Services::default();
        let service = create_test_service();
        let _ = services.create_item(service.clone()).await;

        let result = services.delete_fields(Key::Pk(&service.id), [String::from("name")].into()).await;
        assert!(matches!(result, Err(Error::UnsupportedOperation)), 
                "Deleting fields should return UnsupportedOperation");
    }
}
