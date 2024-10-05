use crate::ports::output::database::{Table, Result}; // Importing necessary traits and types
use crate::domain::types::User;
use std::collections::HashMap;
use bson::oid::ObjectId;
use std::sync::RwLock;


/// A struct representing a collection of users stored in memory.
pub struct Users {
    emails: RwLock<HashMap<String, ObjectId>>,
    users: RwLock<HashMap<ObjectId, User>>,
}

#[cfg(test)]
mod tests {
    use super::Users;
    use crate::domain::types::User;
    use crate::ports::output::database::Table;
    use bson::oid::ObjectId;
    use tokio;

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
        let emails = self.emails.read().map_err(|_| crate::domain::types::Error::Unknown("Failed to acquire read lock on emails".into()))?;
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
        let users = self.users.read().map_err(|_| crate::domain::types::Error::Unknown("Failed to acquire read lock on emails".into()))?;
        match users.get(id) {
            None => Ok(None),
            Some(user) => Ok(Some(user.email.clone()))
        }
    }
}



impl Table for Users {
    type Item = User;
    type Id = ObjectId;

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
            return Err(crate::domain::types::Error::InvalidInput("Email already exists".into()));
        }

        if let Some(existing_email) = self.email(&user.id).await? {
            if existing_email != user.email && self.exists(&user.email).await? {
                return Err(crate::domain::types::Error::InvalidInput("Email already exists".into()));
            }
        }

        let mut users = self.users.write().map_err(|_| crate::domain::types::Error::Unknown("Failed to acquire write lock on users".into()))?;
        let mut emails = self.emails.write().map_err(|_| crate::domain::types::Error::Unknown("Failed to acquire write lock on emails".into()))?;

        let id = ObjectId::new();
        users.insert(id.clone(), user.clone());
        emails.insert(user.email.clone(), id.clone());

        Ok(id)
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
        let users = self.users.read().map_err(|_| crate::domain::types::Error::Unknown("Failed to acquire read lock on users".into()))?;
        Ok(users.get(id).cloned())
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
    async fn update(&self, user: &Self::Item) -> Result<Self::Id> {
        let mut users = self.users.write().map_err(|_| crate::domain::types::Error::Unknown("Failed to acquire write lock on users".into()))?;
        let mut emails = self.emails.write().map_err(|_| crate::domain::types::Error::Unknown("Failed to acquire write lock on emails".into()))?;

        if let Some(id) = emails.get(&user.email) {
            if id != &user.id {
                return Err(crate::domain::types::Error::InvalidInput("Email already exists".into()));
            }
        }

        users.insert(user.id.clone(), user.clone());
        emails.insert(user.email.clone(), user.id.clone());

        Ok(user.id.clone())
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
    async fn delete(&self, id: &Self::Id) -> Result<Self::Id> {
        let mut users = self.users.write().map_err(|_| crate::domain::types::Error::Unknown("Failed to acquire write lock on users".into()))?;
        let mut emails = self.emails.write().map_err(|_| crate::domain::types::Error::Unknown("Failed to acquire write lock on emails".into()))?;

        if let Some(user) = users.remove(id) {
            emails.remove(&user.email);
            Ok(id.clone())
        } else {
            Err(crate::domain::types::Error::NotFound)
        }
    }
}
