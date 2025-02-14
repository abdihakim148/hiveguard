//! Services collection implementation for the memory database
//!
//! This module provides the implementation for storing and managing service records
//! in memory with thread-safe access and index management.

use super::error::Error;
use crate::domain::types::{Id, Key, Service, Value};
use crate::ports::outputs::database::{CreateItem, DeleteItem, GetItem, Item, UpdateItem};
use std::collections::HashMap;
use std::sync::RwLock as Lock;
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
    pub fn does_not_exist(&self, owner_id: &Id, name: &str) -> Result<(), Error> {
        let owner_index = self.owner_index.read()?;
        if let Some(owner_services) = owner_index.get(owner_id) {
            if owner_services.contains_key(name) {
                return Err(Error::ServiceAlreadyExists);
            }
        }
        Ok(())
    }

    /// Find the service ID for a given owner and name
    pub fn pk(&self, owner_id: &Id, name: &str) -> Result<Option<Id>, Error> {
        let owner_index = self.owner_index.read()?;
        Ok(owner_index
            .get(owner_id)
            .and_then(|services| services.get(name).cloned()))
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
    ) -> Result<Service, Self::Error> {
        let option = match key {
            Key::Pk(pk) | Key::Both((pk, _)) => self.services.read()?.get(pk).cloned(),
            Key::Sk(sk) => {
                let owner_index = self.owner_index.read()?;
                let mut option = None;
                for (owner_id, owner_services) in owner_index.iter() {
                    match owner_services.get(sk) {
                        Some(service_id) => option = self.services.read()?.get(service_id).cloned(),
                        None => (),
                    };
                };
                option
            }
        };

        if let Some(service) = option {
            return Ok(service)
        }
        Err(Error::ServiceNotFound)
    }
}

impl UpdateItem<Service> for Services {
    type Error = Error;

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
        mut map: HashMap<String, Value>,
    ) -> Result<Service, Self::Error> {
        let service_id = match key {
            Key::Pk(pk) => pk.clone(),
            Key::Both((pk, _)) => pk.clone(),
            Key::Sk(sk) => {
                let owner_index = self.owner_index.read()?;
                owner_index
                    .iter()
                    .find_map(|(_, services)| services.get(sk).cloned())
                    .ok_or(Error::ServiceNotFound)?
            }
        };

        let mut services = self.services.write()?;
        let mut service = services.get_mut(&service_id).ok_or(Error::ServiceNotFound)?.clone();

        // Update basic fields
        if let Some(value) = map.remove("new_name") {
            let new_name: String = value.try_into()?;

            // Validate old_name matches the current service name
            if let Some(old_name_value) = map.get("old_name") {
                let old_name: String = old_name_value.clone().try_into()?;
                if old_name != service.name {
                    return Err(Error::ServiceNotFound);
                }
            }

            // Update owner index
            let mut owner_index = self.owner_index.write()?;
            if let Some(owner_services) = owner_index.get_mut(&service.owner_id) {
                owner_services.remove(&service.name);
                owner_services.insert(new_name.clone(), service.id.clone());
            }

            service.name = new_name;
        }

        // Update other fields
        if let Some(value) = map.remove("client_secret") {
            service.client_secret = value.try_into()?;
        }
        if let Some(value) = map.remove("redirect_uris") {
            service.redirect_uris = value.try_into()?;
        }
        if let Some(value) = map.remove("scopes") {
            service.scopes = value.try_into()?;
        }
        if let Some(value) = map.remove("grant_types") {
            service.grant_types = value.try_into()?;
        }
        if let Some(value) = map.remove("token_expiry") {
            let (seconds,): (i64,) = value.try_into()?;
            service.token_expiry = Some(Duration::seconds(seconds));
        }

        services.insert(service_id, service.clone());

        Ok(service)
    }
}

impl DeleteItem<Service> for Services {
    type Error = Error;

    async fn delete_item(
        &self,
        key: Key<&<Service as Item>::PK, &<Service as Item>::SK>,
    ) -> Result<(), Self::Error> {
        let service_id = match key {
            Key::Pk(pk) => *pk,
            Key::Both((pk, sk)) => {
                let owner_index = self.owner_index.read()?;
                if let Some(owner_services) = owner_index.get(pk) {
                    if let Some(service_id) = owner_services.get(sk) {
                        service_id.clone()
                    } else {
                        return Err(Error::ServiceNotFound);
                    }
                } else {
                    return Err(Error::ServiceNotFound);
                }
            }
            Key::Sk(sk) => {
                let owner_index = self.owner_index.read()?;
                let service_id = owner_index
                    .iter()
                    .find_map(|(_, owner_services)| owner_services.get(sk).cloned())
                    .ok_or(Error::ServiceNotFound)?;
                service_id
            }
        };

        // Remove from services
        let mut services = self.services.write()?;
        let service = services.remove(&service_id).ok_or(Error::ServiceNotFound)?;

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
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), service);
    }

    #[tokio::test]
    async fn test_create_duplicate_service_name_same_owner() {
        let services = Services::default();
        let service1 = create_test_service();
        let mut service2 = service1.clone();

        let _ = services.create_item(service1).await;
        let result = services.create_item(service2).await;
        assert!(matches!(result, Err(Error::ServiceAlreadyExists)));
    }

    #[tokio::test]
    async fn test_create_service_with_same_name_different_owner() {
        let services = Services::default();
        let mut service1 = create_test_service();
        let mut service2 = create_test_service();
        service2.owner_id = Id(ObjectId::new());

        let _ = services.create_item(service1.clone()).await;
        let result = services.create_item(service2.clone()).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_service_by_id() {
        let services = Services::default();
        let service = create_test_service();
        let _ = services.create_item(service.clone()).await;

        let result = services.get_item(Key::Pk(&service.id)).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), service);
    }

    #[tokio::test]
    async fn test_get_service_by_name() {
        let services = Services::default();
        let service = create_test_service();
        let _ = services.create_item(service.clone()).await;

        let key = Key::Sk(&service.name);

        let result = services.get_item(key).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), service);
    }

    #[tokio::test]
    async fn test_update_service() {
        let services = Services::default();
        let mut service = create_test_service();
        let _ = services.create_item(service.clone()).await;

        service.name = "Updated Service".to_string();
        let result = services
            .update_item(Key::Pk(&service.owner_id), service.clone())
            .await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), service);
    }

    #[tokio::test]
    async fn test_update_service_with_existing_name_same_owner() {
        let services = Services::default();
        let service1 = create_test_service();
        let mut service2 = service1.clone();

        let _ = services.create_item(service1.clone()).await;
        let _ = services.create_item(service2.clone()).await;

        // Try to update service2 with service1's name
        let mut updated_service2 = service2.clone();
        updated_service2.name = service1.name;

        let result = services
            .update_item(Key::Pk(&service2.owner_id), updated_service2)
            .await;
        assert!(matches!(result, Err(Error::ServiceAlreadyExists)));
    }

    #[tokio::test]
    async fn test_delete_service() {
        let services = Services::default();
        let service = create_test_service();
        let _ = services.create_item(service.clone()).await;

        let result = services.delete_item(Key::Pk(&service.id)).await;
        assert!(result.is_ok());

        let get_result = services.get_item(Key::Pk(&service.id)).await;
        assert!(get_result.is_err());
    }
}
