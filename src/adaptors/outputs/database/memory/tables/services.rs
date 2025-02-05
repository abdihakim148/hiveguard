use crate::domain::types::{Either, Key, Value, Service, Error, Id};
use crate::ports::outputs::database::{Table, Item};
use std::collections::{HashMap, HashSet};
use std::sync::RwLock;
use chrono::Duration;

/// The `Services` struct represents a table of services with primary and secondary indices.
/// The primary index is a map from the service's secondary key (SK) to the service itself.
/// The secondary index is a map from the service's primary key (PK) to a set of secondary keys (SK).
#[derive(Default, Debug)]
pub struct Services {
    primary: RwLock<HashMap<<Service as Item>::SK, Service>>,
    secondary: RwLock<HashMap<<Service as Item>::PK, HashSet<<Service as Item>::SK>>>,
}


impl Table for Services {
    type Error = Error;
    type Item = Service;
    type Map = HashMap<String, Value>;
    const NAME: &'static str = "Services";

    /// Creates a new instance of the `Services` table.
    ///
    /// # Returns
    ///
    /// * `Result<Self, Self::Error>` - A new instance of `Services` or an error.
    async fn new() -> Result<Self, Self::Error> {
        Ok(Services::default())
    }

    /// Inserts a new service into the table.
    ///
    /// # Arguments
    ///
    /// * `item` - A reference to the service to be inserted.
    ///
    /// # Returns
    ///
    /// * `Result<<Self::Item as Item>::PK, Self::Error>` - The primary key of the inserted service or an error.
    ///
    /// In this context, PK (Primary Key) is the `owner_id` and SK (Secondary Key) is the `id`.
    async fn create(&self, item: &Self::Item) -> Result<<Self::Item as Item>::PK, Self::Error> {
        let (pk, sk) = (&item.owner_id, &item.id);
        let (mut secondary, mut primary) = (self.secondary.write()?, self.primary.write()?);

        if let Some(_) = primary.get(sk) {
            return Err(Error::Conflict(Self::Item::NAME));
        }

        primary.insert(item.id.clone(), item.clone());
        secondary.entry(pk.clone()).or_insert_with(HashSet::new).insert(sk.clone());
        Ok(item.id.clone())
    }

    /// Retrieves a service from the table using a key.
    ///
    /// # Arguments
    ///
    /// * `key` - A key that can be either a primary key or a combination of primary and secondary keys.
    ///
    /// # Returns
    ///
    /// * `Result<Option<Self::Item>, Self::Error>` - The service if found, or None if not found.
    ///
    /// The SK is not used here because the SK represents the `owner_id`, and an owner can have
    /// multiple services. The function is designed to retrieve a single service, which is
    /// uniquely identified by the primary key (`id`).
    async fn get(&self, key: Key<&<Self::Item as Item>::PK, &<Self::Item as Item>::SK>) -> Result<Option<Self::Item>, Self::Error> {
        let pk = match key {
            Key::Pk(pk) => *pk,
            Key::Sk(sk) => return Ok(None),
            Key::Both((pk, _)) => *pk,
        };
        Ok(self.primary.read()?.get(&pk).cloned())
    }

    /// Retrieves multiple services associated with a primary key.
    ///
    /// # Arguments
    ///
    /// * `key` - An `Either` type that can be a primary key or a secondary key.
    ///
    /// # Returns
    ///
    /// * `Result<Option<Vec<Self::Item>>, Self::Error>` - A vector of services if found, or None if not found.
    ///
    /// In this context, PK is the `owner_id` and SK is not used.
    ///
    /// The `key` parameter is used to specify the primary key (`owner_id`) to retrieve all associated services.
    /// If the `key` is a secondary key (`id`), it is ignored because the method is designed to retrieve
    /// multiple services associated with a single `owner_id`. The secondary key is not applicable in this context
    /// as it would only point to a single service.
    async fn get_many(&self, key: Either<&<Self::Item as Item>::PK, &<Self::Item as Item>::SK>) -> Result<Option<Vec<Self::Item>>, Self::Error> {
        let pk = match key {
            Either::Left(pk) => pk,
            Either::Right(sk) => return Ok(None),
        };
        let secondary = self.secondary.read()?;
        let set = match secondary.get(pk) {
            Some(set) => set,
            None => return Ok(None),
        };
        let mut items = Vec::new();
        for sk in set {
            if let Some(item) = self.primary.read()?.get(sk) {
                items.push(item.clone());
            }
        }
        Ok(Some(items))
    }

    /// Updates specific fields of a service identified by its primary key.
    ///
    /// # Arguments
    ///
    /// * `id` - The primary key of the service to be updated.
    /// * `map` - A map containing the fields to be updated.
    ///
    /// # Returns
    ///
    /// * `Result<Self::Item, Self::Error>` - The updated service or an error.
    async fn patch(&self, id: &<Self::Item as Item>::PK, mut map: Self::Map) -> Result<Self::Item, Self::Error> {
        let key = Key::Pk(id);
        if let Some(service) = self.get(key).await? {
            let id = *id;
            let name = map.remove("name").and_then(|v| v.try_into().ok()).unwrap_or(service.name.clone());
            let client_secret = map.remove("client_secret").and_then(|v| v.try_into().ok()).unwrap_or(service.client_secret.clone());
            let redirect_uris = map.remove("redirect_uris").and_then(|v| v.try_into().ok()).unwrap_or(service.redirect_uris.clone());
            let scopes = map.remove("scopes").and_then(|v| v.try_into().ok()).unwrap_or(service.scopes.clone());
            let grant_types = map.remove("grant_types").and_then(|v| v.try_into().ok()).unwrap_or(service.grant_types.clone());
            let token_expiry = map.remove("token_expiry").and_then(|v| TryInto::<Duration>::try_into(v).ok()).or(service.token_expiry);

            let updated_service = Service {
                owner_id: service.owner_id.clone(),
                id,
                name,
                client_secret,
                redirect_uris,
                scopes,
                grant_types,
                token_expiry,
            };

            self.update(&updated_service).await?;
            return Ok(updated_service);
        }
        Err(Error::NotFound(Self::Item::NAME))
    }

    /// Updates a service in the table.
    ///
    /// # Arguments
    ///
    /// * `item` - A reference to the service to be updated.
    ///
    /// # Returns
    ///
    /// * `Result<(), Self::Error>` - An empty result or an error.
    async fn update(&self, item: &Self::Item) -> Result<(), Self::Error> {
        self.primary.write()?.insert(item.id, item.clone());
        Ok(())
    }

    /// Deletes a service from the table using its primary key.
    ///
    /// # Arguments
    ///
    /// * `id` - The primary key of the service to be deleted.
    ///
    /// # Returns
    ///
    /// * `Result<(), Self::Error>` - An empty result or an error.
    async fn delete(&self, id: &<Self::Item as Item>::PK) -> Result<(), Self::Error> {
        let (mut secondary, mut primary) = (self.secondary.write()?, self.primary.write()?);
        if let Some(service) = primary.remove(id) {
            if let Some(set) = secondary.get_mut(&service.owner_id) {
                set.remove(id);
                if set.is_empty() {
                    secondary.remove(&service.owner_id);
                }
            }
            Ok(())
        } else {
            Err(Error::NotFound(Self::Item::NAME))
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::types::{Id, Service, Scope, GrantType};
    use std::collections::HashMap;
    use chrono::Duration;

    #[tokio::test]
    async fn test_create_service() {
        let services = Services::new().await.unwrap();
        let service = Service {
            owner_id: Id::default(),
            id: Id::default(),
            name: "Test Service".to_string(),
            client_secret: "secret".to_string(),
            redirect_uris: vec!["http://example.com/callback".to_string()],
            scopes: vec![Scope::default()],
            grant_types: vec![GrantType::AuthorizationCode],
            token_expiry: Some(Duration::minutes(60)),
        };

        let result = services.create(&service).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_service() {
        let services = Services::new().await.unwrap();
        let service = Service {
            owner_id: Id::default(),
            id: Id::default(),
            name: "Test Service".to_string(),
            client_secret: "secret".to_string(),
            redirect_uris: vec!["http://example.com/callback".to_string()],
            scopes: vec![Scope::default()],
            grant_types: vec![GrantType::AuthorizationCode],
            token_expiry: Some(Duration::minutes(60)),
        };

        services.create(&service).await.unwrap();
        let result = services.get(Key::Pk(&service.id)).await.unwrap();
        assert_eq!(result, Some(service));
    }

    #[tokio::test]
    async fn test_get_many_services() {
        let services = Services::new().await.unwrap();
        let owner_id = Id::default();
        let service1 = Service {
            owner_id,
            id: Id::default(),
            name: "Service 1".to_string(),
            client_secret: "secret1".to_string(),
            redirect_uris: vec!["http://example1.com/callback".to_string()],
            scopes: vec![Scope::default()],
            grant_types: vec![GrantType::AuthorizationCode],
            token_expiry: Some(Duration::minutes(60)),
        };
        let service2 = Service {
            owner_id,
            id: Id::default(),
            name: "Service 2".to_string(),
            client_secret: "secret2".to_string(),
            redirect_uris: vec!["http://example2.com/callback".to_string()],
            scopes: vec![Scope::default()],
            grant_types: vec![GrantType::AuthorizationCode],
            token_expiry: Some(Duration::minutes(60)),
        };

        services.create(&service1).await.unwrap();
        services.create(&service2).await.unwrap();
        let result = services.get_many(Either::Left(&owner_id)).await.unwrap();
        assert_eq!(result.unwrap().len(), 2);
    }

    #[tokio::test]
    async fn test_patch_service() {
        let services = Services::new().await.unwrap();
        let service = Service {
            owner_id: Id::default(),
            id: Id::default(),
            name: "Test Service".to_string(),
            client_secret: "secret".to_string(),
            redirect_uris: vec!["http://example.com/callback".to_string()],
            scopes: vec![Scope::default()],
            grant_types: vec![GrantType::AuthorizationCode],
            token_expiry: Some(Duration::minutes(60)),
        };

        services.create(&service).await.unwrap();
        let mut map = HashMap::new();
        map.insert("name".to_string(), Value::String("Updated Service".to_string()));
        let updated_service = services.patch(&service.id, map).await.unwrap();
        assert_eq!(updated_service.name, "Updated Service");
    }

    #[tokio::test]
    async fn test_update_service() {
        let services = Services::new().await.unwrap();
        let mut service = Service {
            owner_id: Id::default(),
            id: Id::default(),
            name: "Test Service".to_string(),
            client_secret: "secret".to_string(),
            redirect_uris: vec!["http://example.com/callback".to_string()],
            scopes: vec![Scope::default()],
            grant_types: vec![GrantType::AuthorizationCode],
            token_expiry: Some(Duration::minutes(60)),
        };

        services.create(&service).await.unwrap();
        service.name = "Updated Service".to_string();
        services.update(&service).await.unwrap();
        let result = services.get(Key::Pk(&service.id)).await.unwrap();
        assert_eq!(result.unwrap().name, "Updated Service");
    }

    #[tokio::test]
    async fn test_delete_service() {
        let services = Services::new().await.unwrap();
        let service = Service {
            owner_id: Id::default(),
            id: Id::default(),
            name: "Test Service".to_string(),
            client_secret: "secret".to_string(),
            redirect_uris: vec!["http://example.com/callback".to_string()],
            scopes: vec![Scope::default()],
            grant_types: vec![GrantType::AuthorizationCode],
            token_expiry: Some(Duration::minutes(60)),
        };

        services.create(&service).await.unwrap();
        services.delete(&service.id).await.unwrap();
        let result = services.get(Key::Pk(&service.id)).await.unwrap();
        assert!(result.is_none());
    }
}
