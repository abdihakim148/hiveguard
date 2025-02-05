/// This module defines the `Resources` table which implements the `Table` trait.
/// It provides asynchronous methods to create, retrieve, update, and delete resources
/// in a memory-based database using primary and secondary indices.

use crate::domain::types::{Either, Key, Value, Resource, Error};
use crate::ports::outputs::database::{Table, Item};
use std::collections::HashMap;
use std::collections::HashSet;
use std::sync::RwLock;

/// The `Resources` struct represents a table of resources with primary and secondary indices.
/// The primary index is a map from the resource's secondary key (SK) to the resource itself.
/// The secondary index is a map from the resource's primary key (PK) to a set of secondary keys (SK).
#[derive(Default, Debug)]
pub struct Resources {
    primary: RwLock<HashMap<<Resource as Item>::SK, Resource>>,
    secondary: RwLock<HashMap<<Resource as Item>::PK, HashSet<<Resource as Item>::SK>>>
}


impl Table for Resources {
    type Error = Error;
    type Item = Resource;
    type Map = HashMap<String, Value>;
    const NAME: &'static str = "Resources";
    
    /// Creates a new instance of the `Resources` table.
    ///
    /// # Returns
    ///
    /// * `Result<Self, Self::Error>` - A new instance of `Resources` or an error.
    async fn new() -> Result<Self, Self::Error> {
        Ok(Default::default())
    }

    /// Inserts a new resource into the table.
    ///
    /// In this context, PK (Primary Key) is the `owner_id` and SK (Secondary Key) is the `id`.
    ///
    /// # Arguments
    ///
    /// * `item` - A reference to the resource to be inserted.
    ///
    /// # Returns
    ///
    /// * `Result<<Self::Item as Item>::PK, Self::Error>` - The primary key of the inserted resource or an error.
    async fn create(&self, item: &Self::Item) -> Result<<Self::Item as Item>::PK, Self::Error> {
        // Extract primary and secondary keys from the item.
        // Here, PK is `owner_id` and SK is `id`.
        let (pk, sk) = (&item.owner_id, &item.id);
        let (mut secondary, mut primary) = (self.secondary.write()?, self.primary.write()?);
        // Check if the resource already exists in the primary index.
        if let Some(_) = primary.get(sk) {
            // Return a conflict error if the resource already exists.
            return Err(Error::Conflict(Self::Item::NAME))
        }
        // Check if the primary key exists in the secondary index.
        match secondary.get_mut(pk) {
            // If it exists, check if the secondary key is already associated with the primary key.
            Some(set) => {
                if set.contains(sk) {
                    return Err(Error::Conflict(Self::Item::NAME))
                }
                set.insert(*sk);
            },
            None => { // If the primary key does not exist, create a new set for the secondary key.
                let set = [*sk].into();
                secondary.insert(*pk, set);
            }
        }
        primary.insert(*sk, item.clone());
        Ok(*sk)
    }

    /// Retrieves a resource from the table using a key.
    ///
    /// In this context, PK is the `id` and SK is not used.
    ///
    /// The SK is not used here because the SK represents the `owner_id`, and an owner can have
    /// multiple resources. The function is designed to retrieve a single resource, which is
    /// uniquely identified by the primary key (`id`). Using SK would not be feasible as it
    /// would require selecting one resource among many, which is not deterministic with a
    /// HashMap. If needed, an index could be added to retrieve the first element, but this
    /// is not currently supported.
    ///
    /// # Arguments
    ///
    /// * `key` - A key that can be either a primary key or a combination of primary and secondary keys.
    ///
    /// # Returns
    ///
    /// * `Result<Option<Self::Item>, Self::Error>` - The resource if found, or None if not found.
    async fn get(&self, key: Key<&<Self::Item as Item>::PK, &<Self::Item as Item>::SK>) -> Result<Option<Self::Item>, Self::Error> {
        let pk = match key {
            Key::Pk(pk) => *pk,
            Key::Sk(sk) => return Ok(None),
            Key::Both((pk, _)) => *pk,
        };
        Ok(self.primary.read()?.get(&pk).cloned())
    }

    /// Retrieves multiple resources associated with a primary key.
    ///
    /// In this context, PK is the `owner_id` and SK is not used.
    ///
    /// The SK is not used here because the method is designed to retrieve all resources
    /// associated with a particular `owner_id`. The `owner_id` serves as the primary key
    /// for grouping resources, and the SK is not needed for this operation.
    ///
    /// # Arguments
    ///
    /// * `key` - An `Either` type that can be a primary key or a secondary key.
    ///
    /// # Returns
    ///
    /// * `Result<Option<Vec<Self::Item>>, Self::Error>` - A vector of resources if found, or None if not found.
    async fn get_many(&self, key: Either<&<Self::Item as Item>::PK, &<Self::Item as Item>::SK>) -> Result<Option<Vec<Self::Item>>, Self::Error> {
        let pk = match key {
            Either::Left(pk) => pk,
            Either::Right(sk) => return Ok(None)
        };
        let secondary = self.secondary.read()?;
        let set = match secondary.get(pk) {
            Some(set) => set,
            None => return Ok(None)
        };
        let mut items = Vec::new();
        for sk in set {
            if let Some(item) = self.primary.read()?.get(sk) {
                items.push(item.clone());
            }
        }
        Ok(Some(items))
    }

    /// Updates specific fields of a resource identified by its primary key.
    ///
    /// In this context, PK is the `id`.
    ///
    /// # Arguments
    ///
    /// * `id` - The primary key of the resource to be updated.
    /// * `map` - A map containing the fields to be updated.
    ///
    /// # Returns
    ///
    /// * `Result<Self::Item, Self::Error>` - The updated resource or an error.
    async fn patch(&self, id: &<Self::Item as Item>::PK, mut map: Self::Map) -> Result<Self::Item, Self::Error> {
        let key = Key::Pk(id);
        if let Some(resource) = self.get(key).await? {
            let id = *id;
            let owner_id = match map.remove("owner_id") {
                Some(value) => value.try_into()?,
                None => resource.owner_id.clone()
            };
            let name = match map.remove("name") {
                Some(value) => value.try_into()?,
                None => resource.name.clone()
            };
            let url = match map.remove("url") {
                Some(value) => Some(value.try_into()?),
                None => resource.url.clone()
            };
            let item = Resource{id, owner_id, name, url};
            self.update(&item).await?;
            return Ok(item);
        }
        Err(Error::NotFound(Self::Item::NAME))
    }

    /// Updates a resource in the table.
    ///
    /// In this context, PK is the `id`.
    ///
    /// # Arguments
    ///
    /// * `item` - A reference to the resource to be updated.
    ///
    /// # Returns
    ///
    /// * `Result<(), Self::Error>` - An empty result or an error.
    async fn update(&self, item: &Self::Item) -> Result<(), Self::Error> {
        self.primary.write()?.insert(item.id, item.clone());
        Ok(())
    }

    /// Deletes a resource from the table using its primary key.
    ///
    /// In this context, PK is the `id`.
    ///
    /// # Arguments
    ///
    /// * `id` - The primary key of the resource to be deleted.
    ///
    /// # Returns
    ///
    /// * `Result<(), Self::Error>` - An empty result or an error.
    async fn delete(&self, id: &<Self::Item as Item>::PK) -> Result<(), Self::Error> {
        if let Some(item) = self.primary.write()?.remove(id) {
            if let Some(set) = self.secondary.write()?.get_mut(&item.owner_id) {
                set.remove(&item.id);
            }
        }
        Ok(())
    }
}



#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::types::{Id, Resource};
    use std::collections::HashMap;

    #[tokio::test]
    async fn test_create_resource() {
        let resources = Resources::new().await.unwrap();
        let resource = Resource {
            id: Id::default(),
            owner_id: Id::default(),
            name: "Test Resource".to_string(),
            url: Some("http://example.com".to_string()),
        };

        let result = resources.create(&resource).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_resource() {
        let resources = Resources::new().await.unwrap();
        let resource = Resource {
            id: Id::default(),
            owner_id: Id::default(),
            name: "Test Resource".to_string(),
            url: Some("http://example.com".to_string()),
        };

        resources.create(&resource).await.unwrap();
        let result = resources.get(Key::Pk(&resource.id)).await.unwrap();
        assert_eq!(result, Some(resource));
    }

    #[tokio::test]
    async fn test_get_many_resources() {
        let resources = Resources::new().await.unwrap();
        let owner_id = Id::default();
        let resource1 = Resource {
            id: Id::default(),
            owner_id,
            name: "Resource 1".to_string(),
            url: Some("http://example1.com".to_string()),
        };
        let resource2 = Resource {
            id: Id::default(),
            owner_id,
            name: "Resource 2".to_string(),
            url: Some("http://example2.com".to_string()),
        };

        resources.create(&resource1).await.unwrap();
        resources.create(&resource2).await.unwrap();
        let result = resources.get_many(Either::Left(&owner_id)).await.unwrap();
        assert_eq!(result.unwrap().len(), 2);
    }

    #[tokio::test]
    async fn test_patch_resource() {
        let resources = Resources::new().await.unwrap();
        let resource = Resource {
            id: Id::default(),
            owner_id: Id::default(),
            name: "Test Resource".to_string(),
            url: Some("http://example.com".to_string()),
        };

        resources.create(&resource).await.unwrap();
        let mut map = HashMap::new();
        map.insert("name".to_string(), Value::String("Updated Resource".to_string()));
        let updated_resource = resources.patch(&resource.id, map).await.unwrap();
        assert_eq!(updated_resource.name, "Updated Resource");
    }

    #[tokio::test]
    async fn test_update_resource() {
        let resources = Resources::new().await.unwrap();
        let mut resource = Resource {
            id: Id::default(),
            owner_id: Id::default(),
            name: "Test Resource".to_string(),
            url: Some("http://example.com".to_string()),
        };

        resources.create(&resource).await.unwrap();
        resource.name = "Updated Resource".to_string();
        resources.update(&resource).await.unwrap();
        let result = resources.get(Key::Pk(&resource.id)).await.unwrap();
        assert_eq!(result.unwrap().name, "Updated Resource");
    }

    #[tokio::test]
    async fn test_delete_resource() {
        let resources = Resources::new().await.unwrap();
        let resource = Resource {
            id: Id::default(),
            owner_id: Id::default(),
            name: "Test Resource".to_string(),
            url: Some("http://example.com".to_string()),
        };

        resources.create(&resource).await.unwrap();
        resources.delete(&resource.id).await.unwrap();
        let result = resources.get(Key::Pk(&resource.id)).await.unwrap();
        assert!(result.is_none());
    }
}
