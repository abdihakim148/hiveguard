//! Users collection implementation for the memory database
//! 
//! This module provides the implementation for storing and managing user records
//! in memory with thread-safe access and index management.

use crate::ports::outputs::database::{Item, CreateItem, GetItem, UpdateItem, DeleteItem, Map};
use crate::domain::types::{User, Contact, Key, EmailAddress, Phone, Value};
use std::collections::{HashMap, HashSet};
use std::sync::RwLock as Lock;
use super::error::Error;

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
    pub users: Lock<HashMap<<User as Item>::PK, User>>,
    
    /// Secondary index mapping email addresses to user IDs
    /// Enables fast lookups of users by their email
    pub emails_index: Lock<HashMap<EmailAddress, <User as Item>::PK>>,
    
    /// Secondary index mapping phone numbers to user IDs
    /// Enables fast lookups of users by their phone number
    pub phones_index: Lock<HashMap<Phone, <User as Item>::PK>>,
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
    pub fn update_indexes(&self, pk: <User as Item>::PK, sk: <User as Item>::SK) -> Result<(), Error> {
        let old_contact = self.users.read()?.get(&pk).map(|user| user.contact.clone());

        let (old_phone, old_email) = match &old_contact {
            None => (None, None),
            Some(Contact::Both(phone, email)) => (Some(phone.clone()), Some(email.clone())),
            Some(Contact::Phone(phone)) => (Some(phone.clone()), None),
            Some(Contact::Email(email)) => (None, Some(email.clone()))
        };

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

    pub fn pk(&self, sk: &<User as Item>::SK) -> Result<Option<<User as Item>::PK>, Error> {
        match sk {
            Contact::Phone(phone) => Ok(self.phones_index.read().map(|index| index.get(phone).cloned())?),
            Contact::Email(email) => Ok(self.emails_index.read().map(|index| index.get(email).cloned())?),
            Contact::Both(phone, _) => Ok(self.phones_index.read().map(|index| index.get(phone).cloned())?)
        }
    }

    pub fn does_not_exist(&self, sk: &<User as Item>::SK) -> Result<(), Error> {
        match sk {
            Contact::Phone(phone) => {
                let phones_index = self.phones_index.read()?;
                if phones_index.contains_key(phone) {
                    return Err(Error::UserWithPhoneExists)
                }
                Ok(())
            },
            Contact::Email(email) => {
                let emails_index = self.emails_index.read()?;
                if emails_index.contains_key(email) {
                    return Err(Error::UserWithEmailExists)
                }
                Ok(())
            },
            Contact::Both(phone, email) => {
                let phones_index = self.phones_index.read()?;
                let emails_index = self.emails_index.read()?;
                
                if phones_index.contains_key(phone) {
                    return Err(Error::UserWithPhoneExists)
                }
                if emails_index.contains_key(email) {
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
    
    async fn get_item(&self, key: Key<&<User as Item>::PK, &<User as Item>::SK>) -> Result<User, Self::Error> {
        let option = match key {
            Key::Pk(pk) => self.users.read()?.get(pk).cloned(),
            Key::Both((pk, _)) => self.users.read()?.get(pk).cloned(),
            Key::Sk(sk) => {
                if let Some(pk) = self.pk(sk)? {
                    self.users.read()?.get(&pk).cloned()
                } else {
                    None
                }
            }
        };
        if let Some(user) = option {
            return Ok(user)
        }
        Err(Error::UserNotFound)
    }
}

impl UpdateItem<User> for Users {
    type Error = Error;
    type Update = Map;

    async fn update_item(&self, _: Key<&<User as Item>::PK, &<User as Item>::SK>, user: User) -> Result<User, Self::Error> {
        // Update indexes for new user
        self.update_indexes(user.id, user.contact.clone())?;
        
        // Store updated user
        self.users.write()?.insert(user.id, user.clone());
        Ok(user)
    }

    async fn patch_item(&self, key: Key<&<User as Item>::PK, &<User as Item>::SK>, map: Map) -> Result<User, Self::Error> {
        // First, retrieve the existing user
        let mut user = self.get_item(key.clone()).await?;
        
        // Update basic fields
        if let Some(value) = map.get("username").or_else(|| map.get("user_name")) {
            user.username = value.clone().try_into()?;
        }
        if let Some(value) = map.get("first_name") {
            user.first_name = value.clone().try_into()?;
        }
        if let Some(value) = map.get("last_name") {
            user.last_name = value.clone().try_into()?;
        }
        if let Some(value) = map.get("password") {
            user.password = value.clone().try_into()?;
        }

        // Update contact info if provided
        if map.contains_key("email") || map.contains_key("phone") || 
           map.contains_key("email_verified") || map.contains_key("phone_verified") {
            user.contact = map.try_into()?;
        }

        // Use update_item to handle indexes and storage
        self.update_item(key, user).await
    }

    /// Delete specific fields from a user
    /// 
    /// # Arguments
    /// * `key`: The key to identify the user (by ID or contact info)
    /// * `fields`: List of fields to delete
    /// 
    /// # Behavior
    /// - Only allows deleting email or phone if the user has both contact methods
    /// - Prevents deletion of other fields like username, first_name, etc.
    /// 
    /// # Errors
    /// - Returns `UserNotFound` if the user doesn't exist
    /// - Returns `CannotDeleteContact` if trying to delete the only contact method
    /// - Returns `CannotDeleteField` for attempts to delete non-contact fields
    async fn delete_fields(&self, key: Key<&<User as Item>::PK, &<User as Item>::SK>, mut fields: HashSet<String>) -> Result<User, Self::Error> {
        // Resolve the user ID from the provided key
        let id = match key {
            Key::Both((pk, _)) | Key::Pk(pk) => *pk,
            Key::Sk(sk) => match self.pk(sk)? {
                Some(pk) => pk,
                None => return Err(Error::UserNotFound)
            }
        };

        let mut updated_user = None;

        /// delete email if it was among the fields to be deleted
        if fields.remove(&String::from("email")) {
            let mut users = self.users.write()?;
            let user = users.get_mut(&id).ok_or(Error::UserNotFound)?;
            match &user.contact {
                Contact::Both(phone, _) => user.contact = Contact::Phone(phone.clone()),
                _ => return Err(Error::CannotDeleteContact)
            }
            updated_user = Some(user.clone());
        }

        /// delete phone if it was among the fields to be deleted
        if fields.remove(&String::from("phone")) {
            let mut users = self.users.write()?;
            let user = users.get_mut(&id).ok_or(Error::UserNotFound)?;
            match &user.contact {
                Contact::Both(_, email) => user.contact = Contact::Email(email.clone()),
                _ => return Err(Error::CannotDeleteContact)
            }
            updated_user = Some(user.clone());
        }

        match updated_user {
            Some(user) => Ok(user),
            None => Err(Error::CannotDeleteFields(fields))
        }
    }
}

impl DeleteItem<User> for Users {
    type Error = Error;
    
    async fn delete_item(&self, key: Key<&<User as Item>::PK, &<User as Item>::SK>) -> Result<(), Self::Error> {
        let pk = match key {
            Key::Pk(pk) | Key::Both((pk, _)) => *pk,
            Key::Sk(sk) => self.pk(sk)?.ok_or(Error::UserNotFound)?
        };

        let contact = self.users.read()?.get(&pk)
            .map(|user| user.contact.clone())
            .ok_or(Error::UserNotFound)?;

        self.update_indexes(pk, contact)?;
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
        assert_eq!(result.unwrap(), user);
    }

    #[tokio::test]
    async fn test_get_user_by_email() {
        let users = Users::default();
        let user = create_test_user();
        let _ = users.create_item(user.clone()).await;

        let key = Key::Sk(&user.contact);
        
        let result = users.get_item(key).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), user);
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
        assert!(get_result.is_err());
    }

    /// Test deleting email from a user with both phone and email
    #[tokio::test]
    async fn test_delete_email_field() {
        let users = Users::default();
        let user = create_test_user();
        let _ = users.create_item(user.clone()).await;
        
        // Delete email field
        let result = users.delete_fields(Key::Pk(&user.id), [String::from("email")].into()).await;
        assert!(result.is_ok());
        
        // Verify the user now has only phone contact
        let updated_user = result.unwrap();
        assert!(matches!(updated_user.contact, Contact::Phone(_)));
    }

    /// Test deleting phone from a user with both phone and email
    #[tokio::test]
    async fn test_delete_phone_field() {
        let users = Users::default();
        let user = create_test_user();
        let _ = users.create_item(user.clone()).await;
        
        // Delete phone field
        let result = users.delete_fields(Key::Pk(&user.id), [String::from("phone")].into()).await;
        assert!(result.is_ok());
        
        // Verify the user now has only email contact
        let updated_user = result.unwrap();
        assert!(matches!(updated_user.contact, Contact::Email(_)));
    }

    /// Test attempting to delete email from a user with only email contact
    #[tokio::test]
    async fn test_delete_email_from_single_contact_fails() {
        let users = Users::default();
        let user = User {
            id: Id(ObjectId::new()),
            username: "testuser".to_string(),
            first_name: "Test".to_string(),
            last_name: "User".to_string(),
            password: "hashedpassword".to_string(),
            contact: Contact::Email(EmailAddress::New("test@example.com".parse().unwrap()))
        };
        let _ = users.create_item(user.clone()).await;
        
        // Attempt to delete email
        let result = users.delete_fields(Key::Pk(&user.id), [String::from("email")].into()).await;
        assert!(result.is_err());
        assert!(matches!(result, Err(Error::CannotDeleteContact)));
    }

    /// Test attempting to delete phone from a user with only phone contact
    #[tokio::test]
    async fn test_delete_phone_from_single_contact_fails() {
        let users = Users::default();
        let user = User {
            id: Id(ObjectId::new()),
            username: "testuser".to_string(),
            first_name: "Test".to_string(),
            last_name: "User".to_string(),
            password: "hashedpassword".to_string(),
            contact: Contact::Phone(Phone::New("1234567890".to_string()))
        };
        let _ = users.create_item(user.clone()).await;
        
        // Attempt to delete phone
        let result = users.delete_fields(Key::Pk(&user.id), [String::from("phone")].into()).await;
        assert!(result.is_err());
        assert!(matches!(result, Err(Error::CannotDeleteContact)));
    }

    /// Test attempting to delete non-contact fields
    #[tokio::test]
    async fn test_delete_non_contact_fields_fails() {
        let users = Users::default();
        let user = create_test_user();
        let _ = users.create_item(user.clone()).await;
        
        // Attempt to delete username
        let result = users.delete_fields(Key::Pk(&user.id), [String::from("username")].into()).await;
        assert!(result.is_err());
        assert!(matches!(result, Err(Error::CannotDeleteFields(_))));
    }
}
