//! Users collection implementation for the memory database
//! 
//! This module provides the implementation for storing and managing user records
//! in memory with thread-safe access and index management.

use crate::ports::outputs::database::{Item, CreateItem, GetItem, UpdateItem, DeleteItem};
use crate::domain::types::{User, Contact, Key, EmailAddress, Phone, Value};
use super::error::Error;
use std::collections::HashMap;
use std::sync::RwLock as Lock;

/// Thread-safe, indexed storage for user records
/// 
/// # Indexes
/// - Primary index: User ID -> User record
/// - Secondary indexes:
///   * Email Address -> User ID
///   * Phone Number -> User ID
/// 
/// # Concurrency
/// Uses RwLock to ensure safe concurrent read and write operations
#[derive(Debug, Default)]
pub struct Users {
    /// Primary storage of users, keyed by their unique identifier
    users: Lock<HashMap<<User as Item>::PK, User>>,
    
    /// Secondary index mapping email addresses to user IDs
    /// Enables fast lookups of users by their email
    emails_index: Lock<HashMap<EmailAddress, <User as Item>::PK>>,
    
    /// Secondary index mapping phone numbers to user IDs
    /// Enables fast lookups of users by their phone number
    phones_index: Lock<HashMap<Phone, <User as Item>::PK>>,
}


impl Users {
    /// Updates secondary indexes when a user's contact information changes
    /// 
    /// # Arguments
    /// * `pk`: Primary key (User ID) of the user being updated
    /// * `sk`: New secondary key (Contact information)
    /// 
    /// # Behavior
    /// - Removes old email/phone indexes
    /// - Adds new email/phone indexes
    /// - Handles partial updates (email or phone only)
    fn update_indexes(&self, pk: <User as Item>::PK, sk: <User as Item>::SK) -> Result<(), Error> {
        // Retrieve existing user's contact information
        let (old_phone, old_email) = match self.users.read()?.get(&pk) {
            None => (None, None),
            Some(user) => match &user.contact {
                Contact::Both(phone, email) => (Some(phone.clone()), Some(email.clone())),
                Contact::Phone(phone) => (Some(phone.clone()), None),
                Contact::Email(email) => (None, Some(email.clone()))
            }
        };

        // Determine new contact information
        let (new_phone, new_email) = match sk {
            Contact::Both(phone, email) => (Some(phone), Some(email)),
            Contact::Phone(phone) => (Some(phone), None),
            Contact::Email(email) => (None, Some(email))
        };

        // Update phone number index
        if let Some(phone) = new_phone {
            if let Some(old_phone) = &old_phone {
                self.phones_index.write()?.remove(old_phone);
            }
            self.phones_index.write()?.insert(phone, pk);
        }

        // Update email address index
        if let Some(email) = new_email {
            if let Some(old_email) = &old_email {
                self.emails_index.write()?.remove(old_email);
            }
            self.emails_index.write()?.insert(email, pk);
        }
        Ok(())
    }

    fn pk(&self, sk: &<User as Item>::SK) -> Result<Option<<User as Item>::PK>, Error> {
        match sk {
            Contact::Phone(phone) => Ok(self.phones_index.read()?.get(phone).cloned()),
            Contact::Email(email) => Ok(self.emails_index.read()?.get(email).cloned()),
            Contact::Both(phone, _) => Ok(self.phones_index.read()?.get(phone).cloned())
        }
    }

    fn does_not_exist(&self, sk: &<User as Item>::SK) -> Result<(), Error> {
        match sk {
            Contact::Phone(phone) => {
                if self.phones_index.read()?.contains_key(phone) {
                    return Err(Error::UserWithPhoneExists)
                }
                Ok(())
            },
            Contact::Email(email) => {
                if self.emails_index.read()?.contains_key(email) {
                    return Err(Error::UserWithEmailExists)
                }
                Ok(())
            },
            Contact::Both(phone, email) => {
                if self.phones_index.read()?.contains_key(phone) {
                    return Err(Error::UserWithPhoneExists)
                }
                if self.emails_index.read()?.contains_key(email) {
                    return Err(Error::UserWithEmailExists)
                }
                Ok(())
            }
        }
    }
}


impl CreateItem<User> for Users {
    type Error = Error;
    
    async fn create_item(&self, user: User) -> Result<User, Self::Error> {
        // Check if user with same contact info exists
        self.does_not_exist(&user.contact)?;
        
        // Update indexes
        self.update_indexes(user.id, user.contact.clone())?;
        
        // Store user
        self.users.write()?.insert(user.id, user.clone());
        
        Ok(user)
    }
}


impl GetItem<User> for Users {
    type Error = Error;
    
    async fn get_item(&self, key: Key<&<User as Item>::PK, &<User as Item>::SK>) -> Result<Option<User>, Self::Error> {
        match key {
            Key::Pk(pk) => Ok(self.users.read()?.get(pk).cloned()),
            Key::Both((pk, _)) => Ok(self.users.read()?.get(pk).cloned()),
            Key::Sk(sk) => {
                if let Some(pk) = self.pk(sk)? {
                    Ok(self.users.read()?.get(&pk).cloned())
                } else {
                    Ok(None)
                }
            }
        }
    }
}

impl UpdateItem<User> for Users {
    type Error = Error;

    async fn update_item(&self, _: Key<&<User as Item>::PK, &<User as Item>::SK>, user: User) -> Result<User, Self::Error> {
        // Update indexes for new user
        self.update_indexes(user.id, user.contact.clone())?;
        
        // Store updated user
        self.users.write()?.insert(user.id, user.clone());
        Ok(user)
    }

    async fn patch_item(&self, key: Key<&<User as Item>::PK, &<User as Item>::SK>, mut map: HashMap<String, Value>) -> Result<User, Self::Error> {
        let id = match key {
            Key::Both((pk, _)) | Key::Pk(pk) => *pk,
            Key::Sk(sk) => match self.pk(sk)? {
                Some(pk) => pk,
                None => return Err(Error::UserNotFound)
            }
        };

        let mut users = self.users.write()?;
        let user = users.get_mut(&id).ok_or(Error::UserNotFound)?;
        
        // Update basic fields
        if let Some(value) = map.remove("username").or_else(|| map.remove("user_name")) {
            user.username = value.try_into()?;
        }
        if let Some(value) = map.remove("first_name") {
            user.first_name = value.try_into()?;
        }
        if let Some(value) = map.remove("last_name") {
            user.last_name = value.try_into()?;
        }
        if let Some(value) = map.remove("password") {
            user.password = value.try_into()?;
        }

        // Update contact info if provided
        if map.contains_key("email") || map.contains_key("phone") || 
           map.contains_key("email_verified") || map.contains_key("phone_verified") {
            let contact: Contact = map.try_into()?;
            self.update_indexes(user.id, contact.clone())?;
            user.contact = contact;
        }

        Ok(user.clone())
    }
}

impl DeleteItem<User> for Users {
    type Error = Error;
    
    async fn delete_item(&self, key: Key<&<User as Item>::PK, &<User as Item>::SK>) -> Result<(), Self::Error> {
        let pk = match key {
            Key::Pk(pk) | Key::Both((pk, _)) => *pk,
            Key::Sk(sk) => match self.pk(sk)? {
                Some(pk) => pk,
                None => return Err(Error::UserNotFound)
            }
        };

        // Remove from indexes
        if let Some(user) = self.users.read()?.get(&pk) {
            self.update_indexes(pk, user.contact.clone())?;
        }

        // Remove user
        self.users.write()?.remove(&pk);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::types::Id;
    use bson::oid::ObjectId;
    use super::*;

    /// Helper function to create a test user
    fn create_test_user() -> User {
        User {
            id: Id(ObjectId::new()),
            username: "testuser".to_string(),
            first_name: "Test".to_string(),
            last_name: "User".to_string(),
            password: "hashedpassword".to_string(),
            contact: Contact::Both(
                Phone::New("1234567890".to_string()),
                EmailAddress::New("test@example.com".parse().unwrap())
            )
        }
    }

    #[tokio::test]
    async fn test_create_user() {
        let users = Users::default();
        let user = create_test_user();
        let result = users.create_item(user.clone()).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), user);
    }

    #[tokio::test]
    async fn test_create_duplicate_user() {
        let users = Users::default();
        let user1 = create_test_user();
        let mut user2 = create_test_user();
        user2.id = Id(ObjectId::new());
        
        let _ = users.create_item(user1).await;
        let result = users.create_item(user2).await;
        assert!(matches!(result, Err(Error::UserWithPhoneExists)));
    }

    #[tokio::test]
    async fn test_get_user_by_id() {
        let users = Users::default();
        let user = create_test_user();
        let _ = users.create_item(user.clone()).await;
        
        let result = users.get_item(Key::Pk(&user.id)).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Some(user));
    }

    #[tokio::test]
    async fn test_get_user_by_email() {
        let users = Users::default();
        let user = create_test_user();
        let _ = users.create_item(user.clone()).await;

        let key = Key::Sk(&user.contact);
        
        let result = users.get_item(key).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Some(user));
    }

    #[tokio::test]
    async fn test_update_user() {
        let users = Users::default();
        let mut user = create_test_user();
        let _ = users.create_item(user.clone()).await;
        
        user.username = "updated_user".to_string();
        let result = users.update_item(Key::Pk(&user.id), user.clone()).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), user);
    }

    #[tokio::test]
    async fn test_delete_user() {
        let users = Users::default();
        let user = create_test_user();
        let _ = users.create_item(user.clone()).await;
        
        let result = users.delete_item(Key::Pk(&user.id)).await;
        assert!(result.is_ok());
        
        let get_result = users.get_item(Key::Pk(&user.id)).await;
        assert_eq!(get_result.unwrap(), None);
    }
}
