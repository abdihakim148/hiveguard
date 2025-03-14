//! Verifications collection implementation for the memory database
//! 
//! This module provides the implementation for storing and managing verification records
//! in memory with thread-safe access and index management.

use crate::ports::outputs::database::{Item, CreateItem, GetItem};
use crate::domain::types::{Verification, Key, Either, Phone, EmailAddress, Id};
use std::collections::HashMap;
use std::sync::RwLock as Lock;
use super::error::Error;

/// Thread-safe storage for verification records
/// 
/// # Indexes
/// - Primary index: Contact (Either<Phone, EmailAddress>) -> Verification record
/// - Secondary index: Verification ID -> Contact
/// 
/// # Concurrency
/// Uses RwLock to ensure safe concurrent read and write operations
#[derive(Debug, Default)]
pub struct Verifications {
    /// Primary storage of verifications, keyed by contact information
    pub verifications: Lock<HashMap<<Verification as Item>::PK, Verification>>,
    
    /// Secondary index mapping verification IDs to contact information
    /// Enables lookups of verifications by their ID
    pub ids_index: Lock<HashMap<Id, Either<Phone, EmailAddress>>>,
}

impl Verifications {
    /// Find the contact for a given verification ID
    /// 
    /// # Arguments
    /// * `id` - The ID of the verification
    /// 
    /// # Returns
    /// * `Ok(Some(contact))` if a verification with this ID exists
    /// * `Ok(None)` if no verification with this ID exists
    pub fn contact(&self, id: &Id) -> Result<Option<Either<Phone, EmailAddress>>, Error> {
        Ok(self.ids_index.read()?.get(id).cloned())
    }
}

impl CreateItem<Verification> for Verifications {
    type Error = Error;
    
    async fn create_item(&self, verification: Verification) -> Result<Verification, Self::Error> {
        // Store the verification
        self.verifications.write()?.insert(
            verification.owner_contact.clone(), 
            verification.clone()
        );
        
        // Update the ID index
        self.ids_index.write()?.insert(
            verification.id.clone(), 
            verification.owner_contact.clone()
        );
        
        Ok(verification)
    }
}

impl GetItem<Verification> for Verifications {
    type Error = Error;
    
    async fn get_item(&self, key: Key<&<Verification as Item>::PK, &<Verification as Item>::SK>) -> Result<Verification, Self::Error> {
        let option = match key {
            Key::Pk(pk) => self.verifications.read()?.get(pk).cloned(),
            Key::Sk(sk) => {
                if let Some(contact) = self.contact(sk)? {
                    self.verifications.read()?.get(&contact).cloned()
                } else {
                    None
                }
            },
            Key::Both((pk, _)) => self.verifications.read()?.get(pk).cloned(),
        };
        
        option.ok_or(Error::VerificationNotFound)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::types::{EmailAddress, Phone};
    use bson::oid::ObjectId;
    use chrono::{Utc, Duration};

    /// Helper function to create a test verification for email
    fn create_email_verification() -> Verification {
        let email = EmailAddress::New("test@example.com".parse().unwrap());
        let contact = Either::Right(email);
        Verification {
            owner_contact: contact,
            id: Id(ObjectId::new()),
            code: 123456,
            expires: Utc::now() + Duration::minutes(5),
        }
    }

    /// Helper function to create a test verification for phone
    fn create_phone_verification() -> Verification {
        let phone = Phone::New("1234567890".to_string());
        let contact = Either::Left(phone);
        Verification {
            owner_contact: contact,
            id: Id(ObjectId::new()),
            code: 654321,
            expires: Utc::now() + Duration::minutes(5),
        }
    }

    #[tokio::test]
    async fn test_create_verification() {
        let verifications = Verifications::default();
        let verification = create_email_verification();
        let result = verifications.create_item(verification.clone()).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), verification);
    }

    #[tokio::test]
    async fn test_get_verification_by_contact() {
        let verifications = Verifications::default();
        let verification = create_email_verification();
        let _ = verifications.create_item(verification.clone()).await;
        
        let result = verifications.get_item(Key::Pk(&verification.owner_contact)).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), verification);
    }

    #[tokio::test]
    async fn test_get_verification_by_id() {
        let verifications = Verifications::default();
        let verification = create_phone_verification();
        let _ = verifications.create_item(verification.clone()).await;
        
        let result = verifications.get_item(Key::Sk(&verification.id)).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), verification);
    }

    #[tokio::test]
    async fn test_get_nonexistent_verification() {
        let verifications = Verifications::default();
        let result = verifications.get_item(Key::Pk(&Either::Right(EmailAddress::New("nonexistent@example.com".parse().unwrap())))).await;
        assert!(matches!(result, Err(Error::VerificationNotFound)));
    }
}
