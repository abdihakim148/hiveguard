use crate::ports::outputs::database::{Table, Result}; // Importing necessary traits and types
use crate::domain::types::{User, Value, Error};
use std::collections::HashMap;
use bson::oid::ObjectId;
use std::sync::RwLock;


/// A struct representing a collection of users stored in memory.
pub struct Users {
    emails: RwLock<HashMap<String, ObjectId>>,
    users: RwLock<HashMap<ObjectId, User>>,
}



impl Users {
    /// Checks if a user with the given email exists.
    ///
    /// # Arguments
    ///
    /// * `email` - A string slice that holds the email to check.
    ///
    /// # Returns
    ///
    /// * `Result<bool>` - Returns `Ok(true)` if the email exists, `Ok(false)` otherwise.
    async fn exists(&self, email: &str) -> Result<bool> {
        let emails = self.emails.read().map_err(|_| crate::domain::types::Error::LockError("Failed to acquire read lock on emails".into()))?;
        Ok(emails.contains_key(email))
    }


    /// Retrieves the email associated with a given user ID.
    ///
    /// # Arguments
    ///
    /// * `id` - A reference to the ID of the user whose email is to be retrieved.
    ///
    /// # Returns
    ///
    /// * `Result<Option<String>>` - Returns the email if found, otherwise `None`, wrapped in a `Result`.
    async fn email(&self, id: &ObjectId) -> Result<Option<String>> {
        let users = self.users.read().map_err(|_| crate::domain::types::Error::LockError("Failed to acquire read lock on users".into()))?;
        match users.get(id) {
            None => Ok(None),
            Some(user) => Ok(Some(user.email.clone()))
        }
    }
}



impl Table for Users {
    type Item = User;
    type Id = ObjectId;
    type Map = HashMap<String, Value>;

    const NAME: &'static str = "users";

    /// Creates a new instance of `Users`.
    ///
    /// # Returns
    ///
    /// * `Result<Self>` - Returns a new `Users` instance wrapped in a `Result`.
    async fn new() -> Result<Self> {
        Ok(Users {
            emails: RwLock::new(HashMap::new()),
            users: RwLock::new(HashMap::new()),
        })
    }


    /// Creates a new user.
    ///
    /// # Arguments
    ///
    /// * `user` - A reference to the user item to be created.
    ///
    /// # Returns
    ///
    /// * `Result<Self::Id>` - Returns the ID of the created user wrapped in a `Result`.
    async fn create(&self, user: &Self::Item) -> Result<Self::Id> {
        if self.exists(&user.email).await? {
            return Err(crate::domain::types::Error::EmailAlreadyExists);
        }

        if let Some(existing_email) = self.email(&user.id).await? {
            if existing_email != user.email && self.exists(&user.email).await? {
                return Err(crate::domain::types::Error::EmailAlreadyExists);
            }
        }

        let mut users = self.users.write().map_err(|_| crate::domain::types::Error::LockError("Failed to acquire write lock on users".into()))?;
        let mut emails = self.emails.write().map_err(|_| crate::domain::types::Error::LockError("Failed to acquire write lock on emails".into()))?;

        users.insert(user.id.clone(), user.clone());
        emails.insert(user.email.clone(), user.id.clone());

        Ok(user.id)
    }

    /// Reads a user by ID.
    ///
    /// # Arguments
    ///
    /// * `id` - A reference to the ID of the user to be read.
    ///
    /// # Returns
    ///
    /// * `Result<Option<Self::Item>>` - Returns the user item if found, otherwise `None`, wrapped in a `Result`.
    async fn read(&self, id: &Self::Id) -> Result<Option<Self::Item>> {
        let users = self.users.read().map_err(|_| crate::domain::types::Error::LockError("Failed to acquire read lock on users".into()))?;
        Ok(users.get(id).cloned())
    }


    /// Patches an existing user with the provided map of changes.
    ///
    /// # Arguments
    ///
    /// * `id` - A reference to the ID of the user to be patched.
    /// * `map` - A map containing the fields to be updated and their new values.
    ///
    /// # Returns
    ///
    /// * `Result<Self::Item>` - Returns the updated user item wrapped in a `Result`.
    async fn patch(&self, id: &Self::Id, mut map: Self::Map) -> Result<Self::Item> {
        if let Some(user) = self.read(id).await? {
            let id = *id;
            let username = match map.remove("username") {Some(name) => name.try_into()?, None => user.username};
            let first_name = match map.remove("first_name") {Some(name) => name.try_into()?, None => user.first_name};
            let last_name = match map.remove("last_name") {Some(name) => name.try_into()?, None => user.last_name};
            let email = match map.remove("email") {Some(name) => name.try_into()?, None => user.email};
            let password = match map.remove("password") {Some(name) => name.try_into()?, None => user.password};
            let user = User{id, username, first_name, last_name, email,password};
            let mut users = self.users.write().map_err(|_| crate::domain::types::Error::LockError("Failed to acquire write lock on users".into()))?;
            users.insert(id, user.clone());
            return Ok(user);
        }
        Err(Error::UserNotFound)
    }

    /// Updates an existing user.
    ///
    /// # Arguments
    ///
    /// * `user` - A reference to the user item to be updated.
    ///
    /// # Returns
    ///
    /// * `Result<Self::Id>` - Returns the ID of the updated user wrapped in a `Result`.
    async fn update(&self, user: &Self::Item) -> Result<()> {
        let mut users = self.users.write().map_err(|_| crate::domain::types::Error::LockError("Failed to acquire write lock on users".into()))?;
        let mut emails = self.emails.write().map_err(|_| crate::domain::types::Error::LockError("Failed to acquire write lock on emails".into()))?;

        if let Some(id) = emails.get(&user.email) {
            if id != &user.id {
                return Err(crate::domain::types::Error::EmailAlreadyExists);
            }
        }

        if let Some(existing_user) = users.get_mut(&user.id) {
            emails.remove(&existing_user.email);
            *existing_user = user.clone();
            emails.insert(user.email.clone(), user.id.clone());
        }

        Ok(())
    }

    /// Deletes a user by ID.
    ///
    /// # Arguments
    ///
    /// * `id` - A reference to the ID of the user to be deleted.
    ///
    /// # Returns
    ///
    /// * `Result<Self::Id>` - Returns the ID of the deleted user wrapped in a `Result`.
    async fn delete(&self, id: &Self::Id) -> Result<()> {
        let mut users = self.users.write().map_err(|_| crate::domain::types::Error::LockError("Failed to acquire write lock on users".into()))?;
        let mut emails = self.emails.write().map_err(|_| crate::domain::types::Error::LockError("Failed to acquire write lock on emails".into()))?;

        if let Some(user) = users.remove(id) {
            emails.remove(&user.email);
            Ok(())
        } else {
            Err(crate::domain::types::Error::UserNotFound)
        }
    }
}




#[cfg(test)]
mod tests {
    use super::Users;
    use crate::domain::types::{User, Value};
    use crate::ports::outputs::database::Table;
    use std::collections::HashMap;
    use bson::oid::ObjectId;
    use tokio;

    #[tokio::test]
    async fn test_exists_user_email() {
        let users = Users::new().await.unwrap();
        let user = User {
            id: ObjectId::new(),
            username: "testuser".to_string(),
            first_name: "Test".to_string(),
            last_name: "User".to_string(),
            email: "test@example.com".to_string(),
            password: "password".to_string(),
        };

        // Initially, the email should not exist
        assert_eq!(users.exists(&user.email).await.unwrap(), false);

        // Create the user
        users.create(&user).await.unwrap();

        // Now, the email should exist
        assert_eq!(users.exists(&user.email).await.unwrap(), true);

        // Test with a different email
        assert_eq!(users.exists("nonexistent@example.com").await.unwrap(), false);
    }

    #[tokio::test]
    async fn test_email_retrieval() {
        let users = Users::new().await.unwrap();
        let user = User {
            id: ObjectId::new(),
            username: "testuser".to_string(),
            first_name: "Test".to_string(),
            last_name: "User".to_string(),
            email: "test@example.com".to_string(),
            password: "password".to_string(),
        };

        // Initially, retrieving email by ID should return None
        assert_eq!(users.email(&user.id).await.unwrap(), None);

        // Create the user
        users.create(&user).await.unwrap();

        // Now, retrieving email by ID should return the correct email
        assert_eq!(users.email(&user.id).await.unwrap(), Some(user.email.clone()));

        // Test with a different ID
        let new_id = ObjectId::new();
        assert_eq!(users.email(&new_id).await.unwrap(), None);
    }

    #[tokio::test]
    async fn test_create_user() {
        let users = Users::new().await.unwrap();
        let user = User {
            id: ObjectId::new(),
            username: "testuser".to_string(),
            first_name: "Test".to_string(),
            last_name: "User".to_string(),
            email: "test@example.com".to_string(),
            password: "password".to_string(),
        };

        let result = users.create(&user).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_read_user() {
        let users = Users::new().await.unwrap();
        let user = User {
            id: ObjectId::new(),
            username: "testuser".to_string(),
            first_name: "Test".to_string(),
            last_name: "User".to_string(),
            email: "test@example.com".to_string(),
            password: "password".to_string(),
        };

        let id = users.create(&user).await.unwrap();
        let read_user = users.read(&id).await.unwrap();
        assert_eq!(Some(user), read_user);
    }

    #[tokio::test]
    async fn test_patch_user() {
        let users = Users::new().await.unwrap();
        let user = User {
            id: ObjectId::new(),
            username: "testuser".to_string(),
            first_name: "Test".to_string(),
            last_name: "User".to_string(),
            email: "test@example.com".to_string(),
            password: "password".to_string(),
        };

        // Create the user
        users.create(&user).await.unwrap();

        // Prepare a map with changes
        let mut changes = HashMap::new();
        changes.insert("username".to_string(), Value::String("updateduser".to_string()));
        changes.insert("email".to_string(), Value::String("updated@example.com".to_string()));

        // Patch the user
        let patched_user = users.patch(&user.id, changes).await.unwrap();

        // Verify the changes
        assert_eq!(patched_user.username, "updateduser");
        assert_eq!(patched_user.email, "updated@example.com");
    }

    #[tokio::test]
    async fn test_update_user() {
        let users = Users::new().await.unwrap();
        let mut user = User {
            id: ObjectId::new(),
            username: "testuser".to_string(),
            first_name: "Test".to_string(),
            last_name: "User".to_string(),
            email: "test@example.com".to_string(),
            password: "password".to_string(),
        };

        let id = users.create(&user).await.unwrap();
        user.email = "newemail@example.com".to_string();
        let update_result = users.update(&user).await;
        assert!(update_result.is_ok());

        let updated_user = users.read(&id).await.unwrap();
        assert_eq!(Some(user), updated_user);
    }

    #[tokio::test]
    async fn test_delete_user() {
        let users = Users::new().await.unwrap();
        let user = User {
            id: ObjectId::new(),
            username: "testuser".to_string(),
            first_name: "Test".to_string(),
            last_name: "User".to_string(),
            email: "test@example.com".to_string(),
            password: "password".to_string(),
        };

        let id = users.create(&user).await.unwrap();
        let delete_result = users.delete(&id).await;
        assert!(delete_result.is_ok());

        let deleted_user = users.read(&id).await.unwrap();
        assert!(deleted_user.is_none());
    }
}
