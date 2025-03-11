//! Users collection implementation for the memory database
//! 
//! This module provides the implementation for storing and managing user records
//! in memory with thread-safe access and index management.

use crate::ports::outputs::database::{Item, CreateItem, GetItem, UpdateItem, DeleteItem, Map};
use crate::domain::types::{User, Contact, Key, Value, Either};
#[cfg(any(feature = "email", feature = "contact"))]
use crate::domain::types::EmailAddress;
#[cfg(any(feature = "phone", feature = "contact"))]
use crate::domain::types::Phone;
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
    pub users: Lock<HashMap<<User as Item>::PK, User>>,
    
    /// Secondary index mapping email addresses to user IDs
    /// Enables fast lookups of users by their email
    #[cfg(any(feature = "email", feature = "contact"))]
    pub emails_index: Lock<HashMap<EmailAddress, <User as Item>::PK>>,
    
    /// Secondary index mapping phone numbers to user IDs
    /// Enables fast lookups of users by their phone number
    #[cfg(any(feature = "phone", feature = "contact"))]
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
    /// - Handles feature-gated implementations for different combinations
    #[cfg(any(feature = "contact", all(feature = "email", feature = "phone")))]
    pub fn update_indexes(&self, pk: <User as Item>::PK, sk: Either<Phone, EmailAddress>) -> Result<(), Error> {
        let old_user = self.users.read()?.get(&pk).cloned();

        match sk {
            Either::Left(phone) => {
                #[cfg(feature = "phone")]
                {
                    // Get old phone if exists
                    let old_phone = match &old_user {
                        None => None,
                        #[cfg(feature = "contact")]
                        Some(user) => user.phone.clone(),
                        #[cfg(not(feature = "contact"))]
                        Some(user) => Some(user.phone.clone()),
                    };
                    
                    // Remove old phone from index if exists
                    if let Some(old_phone) = old_phone {
                        self.phones_index.write()?.remove(&old_phone);
                    }
                    
                    // Add new phone to index
                    self.phones_index.write()?.insert(phone, pk);
                }
            },
            Either::Right(email) => {
                #[cfg(feature = "email")]
                {
                    // Get old email if exists
                    let old_email = match &old_user {
                        None => None,
                        #[cfg(feature = "contact")]
                        Some(user) => user.email.clone(),
                        #[cfg(not(feature = "contact"))]
                        Some(user) => Some(user.email.clone()),
                    };
                    
                    // Remove old email from index if exists
                    if let Some(old_email) = old_email {
                        self.emails_index.write()?.remove(&old_email);
                    }
                    
                    // Add new email to index
                    self.emails_index.write()?.insert(email, pk);
                }
            },
        }
        
        Ok(())
    }
    
    #[cfg(all(feature = "email", not(feature = "phone"), not(feature = "contact")))]
    pub fn update_indexes(&self, pk: <User as Item>::PK, email: EmailAddress) -> Result<(), Error> {
        let old_user = self.users.read()?.get(&pk).cloned();
        
        // Get old email if exists
        let old_email = match old_user {
            None => None,
            Some(user) => Some(user.email),
        };
        
        // Remove old email from index if exists
        if let Some(old_email) = old_email {
            self.emails_index.write()?.remove(&old_email);
        }
        
        // Add new email to index
        self.emails_index.write()?.insert(email, pk);
        
        Ok(())
    }
    
    #[cfg(all(feature = "phone", not(feature = "email"), not(feature = "contact")))]
    pub fn update_indexes(&self, pk: <User as Item>::PK, phone: Phone) -> Result<(), Error> {
        let old_user = self.users.read()?.get(&pk).cloned();
        
        // Get old phone if exists
        let old_phone = match old_user {
            None => None,
            Some(user) => Some(user.phone),
        };
        
        // Remove old phone from index if exists
        if let Some(old_phone) = old_phone {
            self.phones_index.write()?.remove(&old_phone);
        }
        
        // Add new phone to index
        self.phones_index.write()?.insert(phone, pk);
        
        Ok(())
    }

    // Different pk methods for different feature combinations
    #[cfg(any(feature = "contact", all(feature = "email", feature = "phone")))]
    pub fn pk(&self, sk: &Either<Phone, EmailAddress>) -> Result<Option<<User as Item>::PK>, Error> {
        return match sk {
            Either::Left(phone) => {
                #[cfg(feature = "phone")]
                return Ok(self.phones_index.read().map(|index| index.get(phone).cloned())?);
                
                #[cfg(not(feature = "phone"))]
                return Ok(None);
            },
            Either::Right(email) => {
                #[cfg(feature = "email")]
                return Ok(self.emails_index.read().map(|index| index.get(email).cloned())?);
                
                #[cfg(not(feature = "email"))]
                return Ok(None);
            },
        };
    }

    #[cfg(all(feature = "email", not(feature = "phone"), not(feature = "contact")))]
    pub fn pk(&self, sk: &EmailAddress) -> Result<Option<<User as Item>::PK>, Error> {
        Ok(self.emails_index.read().map(|index| index.get(sk).cloned())?)
    }

    #[cfg(all(feature = "phone", not(feature = "email"), not(feature = "contact")))]
    pub fn pk(&self, sk: &Phone) -> Result<Option<<User as Item>::PK>, Error> {
        Ok(self.phones_index.read().map(|index| index.get(sk).cloned())?)
    }
    
    #[cfg(not(any(feature = "email", feature = "phone")))]
    pub fn pk(&self, _: &<User as Item>::SK) -> Result<Option<<User as Item>::PK>, Error> {
        Err(Error::UnsupportedOperation)
    }

    // Feature-gated does_not_exist methods for different contact configurations
    #[cfg(any(feature = "contact", all(feature = "email", feature = "phone")))]
    pub fn does_not_exist(&self, sk: &Either<Phone, EmailAddress>) -> Result<(), Error> {
        match sk {
            Either::Left(phone) => {
                #[cfg(feature = "phone")]
                {
                    let phones_index = self.phones_index.read()?;
                    if phones_index.contains_key(phone) {
                        return Err(Error::UserWithPhoneExists)
                    }
                }
                Ok(())
            },
            Either::Right(email) => {
                #[cfg(feature = "email")]
                {
                    let emails_index = self.emails_index.read()?;
                    if emails_index.contains_key(email) {
                        return Err(Error::UserWithEmailExists)
                    }
                }
                Ok(())
            },
        }
    }
    
    #[cfg(all(feature = "email", not(feature = "phone"), not(feature = "contact")))]
    pub fn does_not_exist(&self, email: &EmailAddress) -> Result<(), Error> {
        let emails_index = self.emails_index.read()?;
        if emails_index.contains_key(email) {
            return Err(Error::UserWithEmailExists)
        }
        Ok(())
    }
    
    #[cfg(all(feature = "phone", not(feature = "email"), not(feature = "contact")))]
    pub fn does_not_exist(&self, phone: &Phone) -> Result<(), Error> {
        let phones_index = self.phones_index.read()?;
        if phones_index.contains_key(phone) {
            return Err(Error::UserWithPhoneExists)
        }
        Ok(())
    }
    
    #[cfg(not(any(feature = "email", feature = "phone")))]
    pub fn does_not_exist(&self, _: &<User as Item>::SK) -> Result<(), Error> {
        Err(Error::UnsupportedOperation)
    }
}


#[cfg(feature = "contact")]
impl CreateItem<User> for Users {
    type Error = Error;
    
    async fn create_item(&self, user: User) -> Result<User, Self::Error> {
        // With contact feature, at least one contact method must be provided
        let mut has_contact = false;
        
        #[cfg(feature = "phone")]
        if user.phone.is_some() {
            has_contact = true;
        }
        
        #[cfg(feature = "email")]
        if user.email.is_some() {
            has_contact = true;
        }
        
        if !has_contact {
            return Err(Error::CannotDeleteContact);
        }
        
        // Check and update phone index if provided
        #[cfg(feature = "phone")]
        if let Some(phone) = &user.phone {
            self.does_not_exist(&Either::Left(phone.clone()))?;
            self.update_indexes(user.id.clone(), Either::Left(phone.clone()))?;
        }
        
        // Check and update email index if provided
        #[cfg(feature = "email")]
        if let Some(email) = &user.email {
            self.does_not_exist(&Either::Right(email.clone()))?;
            self.update_indexes(user.id.clone(), Either::Right(email.clone()))?;
        }
        
        // Store user
        self.users.write()?.insert(user.id, user.clone());
        
        Ok(user)
    }
}

#[cfg(all(feature = "email", feature = "phone", not(feature = "contact")))]
impl CreateItem<User> for Users {
    type Error = Error;
    
    async fn create_item(&self, user: User) -> Result<User, Self::Error> {
        // Check if user with same phone exists
        self.does_not_exist(&Either::Left(user.phone.clone()))?;
        
        // Check if user with same email exists
        self.does_not_exist(&Either::Right(user.email.clone()))?;
        
        // Update indexes for phone
        self.update_indexes(user.id.clone(), Either::Left(user.phone.clone()))?;
        
        // Update indexes for email
        self.update_indexes(user.id.clone(), Either::Right(user.email.clone()))?;
        
        // Store user
        self.users.write()?.insert(user.id, user.clone());
        
        Ok(user)
    }
}

#[cfg(all(feature = "email", not(feature = "phone"), not(feature = "contact")))]
impl CreateItem<User> for Users {
    type Error = Error;
    
    async fn create_item(&self, user: User) -> Result<User, Self::Error> {
        // Check if user with same email exists
        self.does_not_exist(&user.email)?;
        
        // Update indexes for email
        self.update_indexes(user.id.clone(), user.email.clone())?;
        
        // Store user
        self.users.write()?.insert(user.id, user.clone());
        
        Ok(user)
    }
}

#[cfg(all(feature = "phone", not(feature = "email"), not(feature = "contact")))]
impl CreateItem<User> for Users {
    type Error = Error;
    
    async fn create_item(&self, user: User) -> Result<User, Self::Error> {
        // Check if user with same phone exists
        self.does_not_exist(&user.phone)?;
        
        // Update indexes for phone
        self.update_indexes(user.id.clone(), user.phone.clone())?;
        
        // Store user
        self.users.write()?.insert(user.id, user.clone());
        
        Ok(user)
    }
}


#[cfg(any(feature = "contact", all(feature = "email", feature = "phone")))]
impl GetItem<User> for Users {
    type Error = Error;
    
    async fn get_item(&self, key: Key<&<User as Item>::PK, &Either<Phone, EmailAddress>>) -> Result<User, Self::Error> {
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

#[cfg(all(feature = "email", not(feature = "phone"), not(feature = "contact")))]
impl GetItem<User> for Users {
    type Error = Error;
    
    async fn get_item(&self, key: Key<&<User as Item>::PK, &EmailAddress>) -> Result<User, Self::Error> {
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

#[cfg(all(feature = "phone", not(feature = "email"), not(feature = "contact")))]
impl GetItem<User> for Users {
    type Error = Error;
    
    async fn get_item(&self, key: Key<&<User as Item>::PK, &Phone>) -> Result<User, Self::Error> {
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

#[cfg(feature = "contact")]
impl UpdateItem<User> for Users {
    type Error = Error;
    type Update = Map;

    async fn update_item(&self, _: Key<&<User as Item>::PK, &Either<Phone, EmailAddress>>, user: User) -> Result<User, Self::Error> {
        // With contact feature, at least one contact method must be provided
        let mut has_contact = false;
        
        #[cfg(feature = "phone")]
        if user.phone.is_some() {
            has_contact = true;
        }
        
        #[cfg(feature = "email")]
        if user.email.is_some() {
            has_contact = true;
        }
        
        if !has_contact {
            return Err(Error::CannotDeleteContact);
        }
        
        // Update indexes for phone if provided
        #[cfg(feature = "phone")]
        if let Some(phone) = &user.phone {
            self.update_indexes(user.id.clone(), Either::Left(phone.clone()))?;
        }
        
        // Update indexes for email if provided
        #[cfg(feature = "email")]
        if let Some(email) = &user.email {
            self.update_indexes(user.id.clone(), Either::Right(email.clone()))?;
        }
        
        // Store updated user
        self.users.write()?.insert(user.id, user.clone());
        Ok(user)
    }
    
    async fn patch_item(&self, key: Key<&<User as Item>::PK, &Either<Phone, EmailAddress>>, mut map: Map) -> Result<User, Self::Error> {
        // First, retrieve the existing user
        let mut user = self.get_item(key.clone()).await?;
        
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

        // Update phone if feature is enabled
        #[cfg(feature = "phone")]
        if map.contains_key("phone_verified") || map.contains_key("phone") {
            let phone = &mut map;
            // For contact feature, handle Option<Phone>
            let phone_value: Result<Phone, _> = phone.try_into();
            if phone_value.is_ok() {
                user.phone = Some(phone_value?);
            } else if map.get("phone").map_or(false, |v| v.is_null()) {
                user.phone = None;
            }
        }

        // Update email if feature is enabled
        #[cfg(feature = "email")]
        if map.contains_key("email") || map.contains_key("email_verified") {
            let email = &mut map;
            // For contact feature, handle Option<EmailAddress>
            let email_value: Result<EmailAddress, _> = email.try_into();
            if email_value.is_ok() {
                user.email = Some(email_value?);
            } else if map.get("email").map_or(false, |v| v.is_null()) {
                user.email = None;
            }
        }
        
        // With contact feature, at least one contact method must be provided
        let mut has_contact = false;
        
        #[cfg(feature = "phone")]
        if user.phone.is_some() {
            has_contact = true;
        }
        
        #[cfg(feature = "email")]
        if user.email.is_some() {
            has_contact = true;
        }
        
        if !has_contact {
            return Err(Error::CannotDeleteContact);
        }

        // Use update_item to handle indexes and storage
        self.update_item(key, user).await
    }
    
    /// Delete specific fields from a user - not supported with contact feature
    async fn delete_fields(&self, _: Key<&<User as Item>::PK, &<User as Item>::SK>, _: &[String]) -> Result<User, Self::Error> {
        Err(Error::CannotDeleteContact)
    }
}

#[cfg(all(feature = "email", feature = "phone", not(feature = "contact")))]
impl UpdateItem<User> for Users {
    type Error = Error;
    type Update = Map;

    async fn update_item(&self, _: Key<&<User as Item>::PK, &Either<Phone, EmailAddress>>, user: User) -> Result<User, Self::Error> {
        // Update indexes for phone
        self.update_indexes(user.id.clone(), Either::Left(user.phone.clone()))?;
        
        // Update indexes for email
        self.update_indexes(user.id.clone(), Either::Right(user.email.clone()))?;
        
        // Store updated user
        self.users.write()?.insert(user.id, user.clone());
        Ok(user)
    }
    
    async fn patch_item(&self, key: Key<&<User as Item>::PK, &Either<Phone, EmailAddress>>, mut map: Map) -> Result<User, Self::Error> {
        // First, retrieve the existing user
        let mut user = self.get_item(key.clone()).await?;
        
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

        // Update phone 
        if map.contains_key("phone_verified") || map.contains_key("phone") {
            let phone = &mut map;
            user.phone = phone.try_into()?;
        }

        // Update email
        if map.contains_key("email") || map.contains_key("email_verified") {
            let email = &mut map;
            user.email = email.try_into()?;
        }

        // Use update_item to handle indexes and storage
        self.update_item(key, user).await
    }
    
    /// Prevents deleting any fields
    async fn delete_fields(&self, _: Key<&<User as Item>::PK, &<User as Item>::SK>, fields: &[String]) -> Result<User, Self::Error> {
        // This method is no longer applicable with the new structure
        Err(Error::CannotDeleteFields(fields.iter().map(|f| f.to_string()).collect()))
    }
}

#[cfg(all(feature = "email", not(feature = "phone"), not(feature = "contact")))]
impl UpdateItem<User> for Users {
    type Error = Error;
    type Update = Map;

    async fn update_item(&self, _: Key<&<User as Item>::PK, &EmailAddress>, user: User) -> Result<User, Self::Error> {
        // Update indexes for email
        self.update_indexes(user.id.clone(), user.email.clone())?;
        
        // Store updated user
        self.users.write()?.insert(user.id, user.clone());
        Ok(user)
    }
    
    async fn patch_item(&self, key: Key<&<User as Item>::PK, &EmailAddress>, mut map: Map) -> Result<User, Self::Error> {
        // First, retrieve the existing user
        let mut user = self.get_item(key.clone()).await?;
        
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

        // Update email
        if map.contains_key("email") || map.contains_key("email_verified") {
            let email = &mut map;
            user.email = email.try_into()?;
        }

        // Use update_item to handle indexes and storage
        self.update_item(key, user).await
    }
    
    /// Prevents deleting any fields
    async fn delete_fields(&self, _: Key<&<User as Item>::PK, &<User as Item>::SK>, fields: &[String]) -> Result<User, Self::Error> {
        // Cannot delete required fields
        Err(Error::CannotDeleteFields(fields.iter().map(|f| f.to_string()).collect()))
    }
}

#[cfg(all(feature = "phone", not(feature = "email"), not(feature = "contact")))]
impl UpdateItem<User> for Users {
    type Error = Error;
    type Update = Map;

    async fn update_item(&self, _: Key<&<User as Item>::PK, &Phone>, user: User) -> Result<User, Self::Error> {
        // Update indexes for phone
        self.update_indexes(user.id.clone(), user.phone.clone())?;
        
        // Store updated user
        self.users.write()?.insert(user.id, user.clone());
        Ok(user)
    }
    
    async fn patch_item(&self, key: Key<&<User as Item>::PK, &Phone>, mut map: Map) -> Result<User, Self::Error> {
        // First, retrieve the existing user
        let mut user = self.get_item(key.clone()).await?;
        
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

        // Update phone
        if map.contains_key("phone_verified") || map.contains_key("phone") {
            let phone = &mut map;
            user.phone = phone.try_into()?;
        }

        // Use update_item to handle indexes and storage
        self.update_item(key, user).await
    }
    
    /// Prevents deleting any fields
    async fn delete_fields(&self, _: Key<&<User as Item>::PK, &<User as Item>::SK>, fields: &[String]) -> Result<User, Self::Error> {
        // Cannot delete required fields
        Err(Error::CannotDeleteFields(fields.iter().map(|f| f.to_string()).collect()))
    }
}

#[cfg(feature = "contact")]
impl DeleteItem<User> for Users {
    type Error = Error;
    
    async fn delete_item(&self, key: Key<&<User as Item>::PK, &Either<Phone, EmailAddress>>) -> Result<(), Self::Error> {
        let pk = match key {
            Key::Pk(pk) | Key::Both((pk, _)) => *pk,
            Key::Sk(sk) => self.pk(sk)?.ok_or(Error::UserNotFound)?
        };

        let user = self.users.read()?.get(&pk)
            .cloned()
            .ok_or(Error::UserNotFound)?;

        // Remove from phone index if present
        #[cfg(feature = "phone")]
        if let Some(phone) = &user.phone {
            self.update_indexes(pk.clone(), Either::Left(phone.clone()))?;
        }
        
        // Remove from email index if present
        #[cfg(feature = "email")]
        if let Some(email) = &user.email {
            self.update_indexes(pk.clone(), Either::Right(email.clone()))?;
        }
        
        self.users.write()?.remove(&pk);
        Ok(())
    }
}

#[cfg(all(feature = "email", feature = "phone", not(feature = "contact")))]
impl DeleteItem<User> for Users {
    type Error = Error;
    
    async fn delete_item(&self, key: Key<&<User as Item>::PK, &Either<Phone, EmailAddress>>) -> Result<(), Self::Error> {
        let pk = match key {
            Key::Pk(pk) | Key::Both((pk, _)) => *pk,
            Key::Sk(sk) => self.pk(sk)?.ok_or(Error::UserNotFound)?
        };

        let user = self.users.read()?.get(&pk)
            .cloned()
            .ok_or(Error::UserNotFound)?;

        // Remove from phone index
        self.update_indexes(pk.clone(), Either::Left(user.phone.clone()))?;
        
        // Remove from email index
        self.update_indexes(pk.clone(), Either::Right(user.email.clone()))?;
        
        self.users.write()?.remove(&pk);
        Ok(())
    }
}

#[cfg(all(feature = "email", not(feature = "phone"), not(feature = "contact")))]
impl DeleteItem<User> for Users {
    type Error = Error;
    
    async fn delete_item(&self, key: Key<&<User as Item>::PK, &EmailAddress>) -> Result<(), Self::Error> {
        let pk = match key {
            Key::Pk(pk) | Key::Both((pk, _)) => *pk,
            Key::Sk(sk) => self.pk(sk)?.ok_or(Error::UserNotFound)?
        };

        let user = self.users.read()?.get(&pk)
            .cloned()
            .ok_or(Error::UserNotFound)?;

        // Remove from email index
        self.update_indexes(pk.clone(), user.email.clone())?;
        
        self.users.write()?.remove(&pk);
        Ok(())
    }
}

#[cfg(all(feature = "phone", not(feature = "email"), not(feature = "contact")))]
impl DeleteItem<User> for Users {
    type Error = Error;
    
    async fn delete_item(&self, key: Key<&<User as Item>::PK, &Phone>) -> Result<(), Self::Error> {
        let pk = match key {
            Key::Pk(pk) | Key::Both((pk, _)) => *pk,
            Key::Sk(sk) => self.pk(sk)?.ok_or(Error::UserNotFound)?
        };

        let user = self.users.read()?.get(&pk)
            .cloned()
            .ok_or(Error::UserNotFound)?;

        // Remove from phone index
        self.update_indexes(pk.clone(), user.phone.clone())?;
        
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
        let mut user = User {
            id: Id(ObjectId::new()),
            username: "testuser".to_string(),
            first_name: "Test".to_string(),
            last_name: "User".to_string(),
            password: "hashedpassword".to_string(),
            #[cfg(all(feature = "phone", not(feature = "contact")))]
            phone: Phone::New("1234567890".to_string()),
            #[cfg(all(feature = "phone", feature = "contact"))]
            phone: Some(Phone::New("1234567890".to_string())),
            #[cfg(all(feature = "email", not(feature = "contact")))]
            email: EmailAddress::New("test@example.com".parse().unwrap()),
            #[cfg(all(feature = "email", feature = "contact"))]
            email: Some(EmailAddress::New("test@example.com".parse().unwrap())),
        };
        
        user
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
    #[cfg(feature = "email")]
    async fn test_get_user_by_email() {
        let users = Users::default();
        let user = create_test_user();
        let _ = users.create_item(user.clone()).await;
        
        // Create the appropriate key based on feature flags
        #[cfg(all(feature = "email", feature = "phone", not(feature = "contact")))]
        let email_key = Either::Right(user.email.clone());
        
        #[cfg(all(feature = "email", feature = "phone", feature = "contact"))]
        let email_key = Either::Right(user.email.clone());
        
        #[cfg(all(feature = "email", not(feature = "phone"), not(feature = "contact")))]
        let email_key = user.email.clone();

        let key = Key::Sk(&email_key);
        
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

    /// Test attempting to delete fields from a user
    #[tokio::test]
    async fn test_delete_fields_fails() {
        let users = Users::default();
        let user = create_test_user();
        let _ = users.create_item(user.clone()).await;
        
        // Attempt to delete email field
        let result = users.delete_fields(Key::Pk(&user.id), &["email".to_string()]).await;
        assert!(result.is_err());
        assert!(matches!(result, Err(Error::CannotDeleteContact)));
        
        // Attempt to delete phone field
        let result = users.delete_fields(Key::Pk(&user.id), &["phone".to_string()]).await;
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
        let result = users.delete_fields(Key::Pk(&user.id), &["username".to_string()]).await;
        assert!(result.is_err());
        // assert!(matches!(result, Err(Error::CannotDeleteField(_))));
    }
}
