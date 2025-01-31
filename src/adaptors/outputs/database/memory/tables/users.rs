use crate::domain::types::{User, Value, EmailAddress, Either};
use crate::ports::outputs::database::{Table, Item}; // Importing necessary traits and types
use std::collections::HashMap;
use super::super::Error;
use bson::oid::ObjectId;
use std::sync::RwLock;


/// A struct representing a collection of users stored in memory.
#[derive(Debug, Default)]
pub struct Users {
    emails: RwLock<HashMap<EmailAddress, ObjectId>>,
    users: RwLock<HashMap<ObjectId, User>>,
}



impl Users {
    const USER: &'static str = "user";
    /// Checks if a user with the given email exists.
    ///
    /// # Arguments
    ///
    /// * `email` - A string slice that holds the email to check.
    ///
    /// # Returns
    ///
    /// * `Result<bool>` - Returns `Ok(true)` if the email exists, `Ok(false)` otherwise.
    async fn exists(&self, email: &EmailAddress) -> Result<bool, Error> {
        let emails = self.emails.read().map_err(|_| Error::LockPoisoned)?;
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
    async fn email(&self, id: &ObjectId) -> Result<Option<EmailAddress>, Error> {
        let users = self.users.read().map_err(|_| Error::LockPoisoned)?;
        match users.get(id) {
            None => Ok(None),
            Some(user) => Ok(Some(user.email.clone()))
        }
    }
}



impl Table for Users {
    type Map = HashMap<String, Value>;
    type Item = User;
    type Error = Error;

    const NAME: &'static str = "users";

    /// Creates a new instance of `Users`.
    ///
    /// # Returns
    ///
    /// * `Result<Self>` - Returns a new `Users` instance wrapped in a `Result`.
    async fn new() -> Result<Self, Self::Error> {
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
    /// * `Result<<User as Item>::PK>` - Returns the ID of the created user wrapped in a `Result`.
    async fn create(&self, user: &User) -> Result<<User as Item>::PK, Self::Error> {
        if self.exists(&user.email).await? {
            return Err(Error::UserWithEmailExists);
        }

        if let Some(existing_email) = self.email(&user.id).await? {
            if existing_email != user.email && self.exists(&user.email).await? {
                return Err(Error::UserWithEmailExists);
            }
        }

        let mut users = self.users.write().map_err(|_| Error::LockPoisoned)?;
        let mut emails = self.emails.write().map_err(|_| Error::LockPoisoned)?;

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
    /// * `Result<Option<User>>` - Returns the user item if found, otherwise `None`, wrapped in a `Result`.
    async fn get(&self, key: Either<&<User as Item>::PK, &<User as Item>::SK>) -> Result<Option<User>, Self::Error> {
        match key {
            Either::Left(id) => {
                let users = self.users.read().map_err(|_| Error::LockPoisoned)?;
                Ok(users.get(id).cloned())
            },
            Either::Right(email) => {
                let emails = self.emails.read().map_err(|_|Error::LockPoisoned)?;
                match emails.get(email) {
                    Some(id) => {
                        let users = self.users.read().map_err(|_| Error::LockPoisoned)?;
                        Ok(users.get(id).cloned())
                    },
                    None => Ok(None)
                }
            }
        }
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
    /// * `Result<User>` - Returns the updated user item wrapped in a `Result`.
    async fn patch(&self, id: &<User as Item>::PK, mut map: Self::Map) -> Result<User, Self::Error> {
        let key = Either::Left(id);
        if let Some(user) = self.get(key).await? {
            let id = *id;
            let username = match map.remove("username") {Some(name) => name.try_into()?, None => user.username};
            let first_name = match map.remove("first_name") {Some(first_name) => first_name.try_into()?, None => user.first_name};
            let last_name = match map.remove("last_name") {Some(name) => name.try_into()?, None => user.last_name};
            let email = match map.remove("email") {Some(name) => name.try_into()?, None => user.email};
            let password = match map.remove("password") {Some(name) => name.try_into()?, None => user.password};
            let user = User{id, username, first_name, last_name, email,password};
            let mut users = self.users.write().map_err(|_| Error::LockPoisoned)?;
            users.insert(id, user.clone());
            return Ok(user);
        }
        Err(Error::NotFound(Self::USER))
    }

    /// Updates an existing user.
    ///
    /// # Arguments
    ///
    /// * `user` - A reference to the user item to be updated.
    ///
    /// # Returns
    ///
    /// * `Result<<User as Item>::PK>` - Returns the ID of the updated user wrapped in a `Result`.
    async fn update(&self, user: &User) -> Result<(), Self::Error> {
        let mut users = self.users.write().map_err(|_| Error::LockPoisoned)?;
        let mut emails = self.emails.write().map_err(|_| Error::LockPoisoned)?;

        if let Some(id) = emails.get(&user.email) {
            if id != &user.id {
                return Err(Error::UserWithEmailExists);
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
    /// * `Result<<User as Item>::PK>` - Returns the ID of the deleted user wrapped in a `Result`.
    async fn delete(&self, id: &<User as Item>::PK) -> Result<(), Self::Error> {
        let mut users = self.users.write().map_err(|_| Error::LockPoisoned)?;
        let mut emails = self.emails.write().map_err(|_| Error::LockPoisoned)?;

        if let Some(user) = users.remove(id) {
            emails.remove(&user.email);
            Ok(())
        } else {
            Err(Error::NotFound(Self::USER))
        }
    }
}




#[cfg(test)]
mod tests {
    use super::Users;
    use crate::domain::types::{User, Value, EmailAddress, Either};
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
            email: EmailAddress::new("test@example.com").unwrap(),
            password: "password".to_string(),
        };

        // Initially, the email should not exist
        assert_eq!(users.exists(&user.email).await.unwrap(), false);

        // Create the user
        users.create(&user).await.unwrap();

        // Now, the email should exist
        assert_eq!(users.exists(&user.email).await.unwrap(), true);

        // Test with a different email
        assert_eq!(users.exists(&EmailAddress::new("nonexistent@example.com").unwrap()).await.unwrap(), false);
    }

    #[tokio::test]
    async fn test_email_retrieval() {
        let users = Users::new().await.unwrap();
        let user = User {
            id: ObjectId::new(),
            username: "testuser".to_string(),
            first_name: "Test".to_string(),
            last_name: "User".to_string(),
            email: EmailAddress::new("test@example.com").unwrap(),
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
            email: EmailAddress::new("test@example.com").unwrap(),
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
            email: EmailAddress::new("test@example.com").unwrap(),
            password: "password".to_string(),
        };

        let id = users.create(&user).await.unwrap();
        let key = Either::Left(&id);
        let read_user = users.get(key).await.unwrap();
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
            email: EmailAddress::new("test@example.com").unwrap(),
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
        assert_eq!(patched_user.email, EmailAddress::new("updated@example.com").unwrap());
    }

    #[tokio::test]
    async fn test_update_user() {
        let users = Users::new().await.unwrap();
        let mut user = User {
            id: ObjectId::new(),
            username: "testuser".to_string(),
            first_name: "Test".to_string(),
            last_name: "User".to_string(),
            email: EmailAddress::new("test@example.com").unwrap(),
            password: "password".to_string(),
        };

        let id = users.create(&user).await.unwrap();
        let key = Either::Left(&id);
        user.email = EmailAddress::new("newemail@example.com").unwrap();
        let update_result = users.update(&user).await;
        assert!(update_result.is_ok());

        let updated_user = users.get(key).await.unwrap();
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
            email: EmailAddress::new("test@example.com").unwrap(),
            password: "password".to_string(),
        };

        let id = users.create(&user).await.unwrap();
        let key = Either::Left(&id);
        let delete_result = users.delete(&id).await;
        assert!(delete_result.is_ok());

        let deleted_user = users.get(key).await.unwrap();
        assert!(deleted_user.is_none());
    }
}
