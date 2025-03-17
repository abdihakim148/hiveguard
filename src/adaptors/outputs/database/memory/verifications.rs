//! Verifications collection implementation for the memory database
//! 
//! This module provides the implementation for storing and managing verification records
//! in memory with thread-safe access and index management.

use crate::ports::outputs::database::{Item, CreateItem, GetItem};
use crate::domain::types::{Verification, Key, Either, Phone, EmailAddress, Id};
use std::collections::HashMap;
use std::sync::{RwLock as Lock, Arc};
use std::time::{Duration, Instant};
use chrono::Utc;
use super::error::Error;
use std::thread;
use std::hash::Hash;
use std::fmt::Debug;

/// Thread-safe storage for verification records with generic ID type
/// 
/// # Type Parameters
/// * `ID` - The type of ID used for verifications, defaults to `Id`
/// 
/// # Indexes
/// - Primary index: Contact (Either<Phone, EmailAddress>) -> Verification record
/// - Secondary index: Verification ID -> Contact
/// 
/// # Concurrency
/// Uses RwLock to ensure safe concurrent read and write operations
/// 
/// # Expiration
/// - Expired verifications are kept for 30 minutes after expiration
/// - Cleanup is performed automatically during operations
#[derive(Debug)]
pub struct Verifications<ID = Id> 
where
    ID: Clone + Hash + PartialEq + Eq + Debug + Send + Sync + 'static
{
    /// Primary storage of verifications, keyed by contact information
    pub verifications: Lock<HashMap<<Verification<ID> as Item>::PK, Verification<ID>>>,
    
    /// Secondary index mapping verification IDs to contact information
    /// Enables lookups of verifications by their ID
    pub ids_index: Lock<HashMap<ID, Either<Phone, EmailAddress>>>,
    
    /// Tracks when expired verifications should be removed
    /// Maps contact information to the time when it should be removed
    expiration_tracker: Lock<HashMap<Either<Phone, EmailAddress>, Instant>>,
    
    /// Grace period after expiration before deletion (30 minutes)
    cleanup_grace_period: Duration,
}

impl<ID> Default for Verifications<ID> 
where
    ID: Clone + Hash + PartialEq + Eq + Debug + Send + Sync + 'static
{
    fn default() -> Self {
        let instance = Self {
            verifications: Lock::new(HashMap::new()),
            ids_index: Lock::new(HashMap::new()),
            expiration_tracker: Lock::new(HashMap::new()),
            cleanup_grace_period: Duration::from_secs(30 * 60), // 30 minutes
        };
        
        // Start background cleanup thread
        instance.start_cleanup_thread();
        
        instance
    }
}

impl<ID> Verifications<ID> 
where
    ID: Clone + Hash + PartialEq + Eq + Debug + Send + Sync + 'static
{
    /// Find the contact for a given verification ID
    /// 
    /// # Arguments
    /// * `id` - The ID of the verification
    /// 
    /// # Returns
    /// * `Ok(Some(contact))` if a verification with this ID exists
    /// * `Ok(None)` if no verification with this ID exists
    pub fn contact(&self, id: &ID) -> Result<Option<Either<Phone, EmailAddress>>, Error> {
        Ok(self.ids_index.read()?.get(id).cloned())
    }
    
    /// Check if a verification is expired
    /// 
    /// # Arguments
    /// * `verification` - The verification to check
    /// 
    /// # Returns
    /// * `true` if the verification has expired
    /// * `false` if the verification is still valid
    fn is_expired(&self, verification: &Verification<ID>) -> bool {
        verification.expires < Utc::now()
    }
    
    /// Mark a verification for cleanup after the grace period
    /// 
    /// # Arguments
    /// * `contact` - The contact information associated with the verification
    async fn mark_for_cleanup(&self, contact: &Either<Phone, EmailAddress>) -> Result<(), Error> {
        let cleanup_time = Instant::now() + self.cleanup_grace_period;
        // Minimize lock holding time
        {
            let mut tracker = self.expiration_tracker.write()?;
            tracker.insert(contact.clone(), cleanup_time);
        }
        Ok(())
    }
    
    /// Remove expired verifications that have passed their grace period
    async fn cleanup_expired(&self) -> Result<(), Error> {
        let now = Instant::now();
        let mut to_remove = Vec::new();
        let mut id_to_remove = Vec::new();
        
        // Find contacts that need to be removed - minimize lock holding time
        {
            let tracker = self.expiration_tracker.read()?;
            for (contact, expiry_time) in tracker.iter() {
                if *expiry_time <= now {
                    to_remove.push(contact.clone());
                }
            }
        }
        
        // Get IDs to remove - separate read lock
        if !to_remove.is_empty() {
            let verifications = self.verifications.read()?;
            for contact in &to_remove {
                if let Some(verification) = verifications.get(contact) {
                    id_to_remove.push(verification.id.clone());
                }
            }
        }
        
        // Remove from each collection separately to minimize lock contention
        if !to_remove.is_empty() {
            // Remove from main storage
            {
                let mut verifications = self.verifications.write()?;
                for contact in &to_remove {
                    verifications.remove(contact);
                }
            }
            
            // Remove from ID index
            {
                let mut ids_index = self.ids_index.write()?;
                for id in &id_to_remove {
                    ids_index.remove(id);
                }
            }
            
            // Remove from expiration tracker
            {
                let mut tracker = self.expiration_tracker.write()?;
                for contact in &to_remove {
                    tracker.remove(contact);
                }
            }
        }
        
        Ok(())
    }
    
    /// Start a background thread to periodically clean up expired verifications
    fn start_cleanup_thread(&self) {
        let verifications = Arc::new(self.clone());
        
        thread::spawn(move || {
            loop {
                thread::sleep(Duration::from_secs(60)); // Check every minute
                
                // Use a separate thread for cleanup to avoid blocking this thread
                let cleanup_verifications = verifications.clone();
                thread::spawn(move || {
                    // Create a runtime for the async cleanup
                    let rt = tokio::runtime::Runtime::new().unwrap();
                    rt.block_on(async {
                        if let Err(e) = cleanup_verifications.cleanup_expired().await {
                            eprintln!("Error during verification cleanup: {}", e);
                        }
                    });
                });
            }
        });
    }
}

impl<ID> Clone for Verifications<ID> 
where
    ID: Clone + Hash + PartialEq + Eq + Debug + Send + Sync + 'static
{
    fn clone(&self) -> Self {
        // Safe unwrap - if the lock is poisoned, we're in a bad state anyway
        let verifications = match self.verifications.read() {
            Ok(guard) => guard.clone(),
            Err(_) => HashMap::new(),
        };
        
        let ids_index = match self.ids_index.read() {
            Ok(guard) => guard.clone(),
            Err(_) => HashMap::new(),
        };
        
        let expiration_tracker = match self.expiration_tracker.read() {
            Ok(guard) => guard.clone(),
            Err(_) => HashMap::new(),
        };
        
        Self {
            verifications: Lock::new(verifications),
            ids_index: Lock::new(ids_index),
            expiration_tracker: Lock::new(expiration_tracker),
            cleanup_grace_period: self.cleanup_grace_period,
        }
    }
}

impl<ID> CreateItem<Verification<ID>> for Verifications<ID> 
where
    ID: Clone + Hash + PartialEq + Eq + Debug + Send + Sync + 'static
{
    type Error = Error;
    
    async fn create_item(&self, verification: Verification<ID>) -> Result<Verification<ID>, Self::Error> {
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

impl<ID> GetItem<Verification<ID>> for Verifications<ID> 
where
    ID: Clone + Hash + PartialEq + Eq + Debug + Send + Sync + 'static
{
    type Error = Error;
    
    async fn get_item(&self, key: Key<&<Verification<ID> as Item>::PK, &<Verification<ID> as Item>::SK>) -> Result<Verification<ID>, Self::Error> {
        // Run cleanup of expired verifications in the background
        let cleanup_verifications = self.clone();
        tokio::spawn(async move {
            if let Err(e) = cleanup_verifications.cleanup_expired().await {
                eprintln!("Error during verification cleanup: {}", e);
            }
        });
        
        // Get the verification
        let verification = match key {
            Key::Pk(pk) => {
                let verifications = self.verifications.read()?;
                verifications.get(pk).cloned()
            },
            Key::Sk(sk) => {
                // First get the contact from the ID
                let contact = {
                    let ids_index = self.ids_index.read()?;
                    ids_index.get(sk).cloned()
                };
                
                // Then get the verification using the contact
                if let Some(contact) = contact {
                    let verifications = self.verifications.read()?;
                    verifications.get(&contact).cloned()
                } else {
                    None
                }
            },
            Key::Both((pk, _)) => {
                let verifications = self.verifications.read()?;
                verifications.get(pk).cloned()
            },
        };
        
        match verification {
            Some(verification) => {
                // Check if verification is expired
                if self.is_expired(&verification) {
                    // Mark for cleanup after grace period - don't block on this
                    let contact = verification.owner_contact.clone();
                    let cleanup_verifications = self.clone();
                    tokio::spawn(async move {
                        if let Err(e) = cleanup_verifications.mark_for_cleanup(&contact).await {
                            eprintln!("Failed to mark verification for cleanup: {}", e);
                        }
                    });
                    Err(Error::VerificationExpired)
                } else {
                    Ok(verification)
                }
            },
            None => Err(Error::VerificationNotFound)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::types::{EmailAddress, Phone};
    use bson::oid::ObjectId;
    use chrono::{Utc, Duration as ChronoDuration};

    /// Helper function to create a test verification for email
    fn create_email_verification() -> Verification<Id> {
        let email = EmailAddress::New("test@example.com".parse().unwrap());
        let contact = Either::Right(email);
        Verification {
            owner_contact: contact,
            id: Id(ObjectId::new()),
            code: 123456,
            expires: Utc::now() + ChronoDuration::minutes(5),
        }
    }

    /// Helper function to create a test verification for phone
    fn create_phone_verification() -> Verification<Id> {
        let phone = Phone::New("1234567890".to_string());
        let contact = Either::Left(phone);
        Verification {
            owner_contact: contact,
            id: Id(ObjectId::new()),
            code: 654321,
            expires: Utc::now() + ChronoDuration::minutes(5),
        }
    }
    
    /// Helper function to create an expired verification
    fn create_expired_verification() -> Verification<Id> {
        let email = EmailAddress::New("expired@example.com".parse().unwrap());
        let contact = Either::Right(email);
        Verification {
            owner_contact: contact,
            id: Id(ObjectId::new()),
            code: 111111,
            expires: Utc::now() - ChronoDuration::minutes(5),
        }
    }

    #[tokio::test]
    async fn test_create_verification() {
        let verifications: Verifications<Id> = Verifications::default();
        let verification = create_email_verification();
        let result = verifications.create_item(verification.clone()).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), verification);
    }

    #[tokio::test]
    async fn test_get_verification_by_contact() {
        let verifications: Verifications<Id> = Verifications::default();
        let verification = create_email_verification();
        let _ = verifications.create_item(verification.clone()).await;
        
        let result = verifications.get_item(Key::Pk(&verification.owner_contact)).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), verification);
    }

    #[tokio::test]
    async fn test_get_verification_by_id() {
        let verifications: Verifications<Id> = Verifications::default();
        let verification = create_phone_verification();
        let _ = verifications.create_item(verification.clone()).await;
        
        let result = verifications.get_item(Key::Sk(&verification.id)).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), verification);
    }

    #[tokio::test]
    async fn test_get_nonexistent_verification() {
        let verifications: Verifications<Id> = Verifications::default();
        let result = verifications.get_item(Key::Pk(&Either::Right(EmailAddress::New("nonexistent@example.com".parse().unwrap())))).await;
        assert!(matches!(result, Err(Error::VerificationNotFound)));
    }
    
    #[tokio::test]
    async fn test_expired_verification() {
        let verifications: Verifications<Id> = Verifications::default();
        let verification = create_expired_verification();
        let _ = verifications.create_item(verification.clone()).await;
        
        let result = verifications.get_item(Key::Pk(&verification.owner_contact)).await;
        assert!(matches!(result, Err(Error::VerificationExpired)));
    }
    
    #[tokio::test]
    async fn test_cleanup_expired_verifications() {
        let verifications: Verifications<Id> = Verifications::default();
        
        // Create an expired verification
        let verification = create_expired_verification();
        let _ = verifications.create_item(verification.clone()).await;
        
        // Mark it for immediate cleanup (override the default grace period)
        {
            let mut tracker = verifications.expiration_tracker.write().unwrap();
            tracker.insert(verification.owner_contact.clone(), Instant::now() - Duration::from_secs(1));
        }
        
        // Trigger cleanup
        let _ = verifications.cleanup_expired().await;
        
        // Verification should be removed
        let result = verifications.get_item(Key::Pk(&verification.owner_contact)).await;
        assert!(matches!(result, Err(Error::VerificationNotFound)));
    }
}
