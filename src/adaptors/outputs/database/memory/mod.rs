//! Memory Database Implementation
//! 
//! This module provides a thread-safe, in-memory database implementation for User entities.
//! 
//! # Features
//! - Uses RwLock for concurrent, safe access to data
//! - Supports CRUD (Create, Read, Update, Delete) operations
//! - Maintains primary and secondary indexes for efficient lookups
//! - Supports indexing by user ID, email, and phone number
//! 
//! # Thread Safety
//! All operations are protected by read-write locks, ensuring safe concurrent access.

mod error;
mod users;

use crate::ports::outputs::database::{Item, CreateItem, GetItem, UpdateItem, DeleteItem};
use crate::domain::types::{User, Key, Value};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::sync::RwLock as Lock;
use error::*;
use users::*;

/// An in-memory database implementation for User entities.
/// 
/// # Concurrency
/// Uses RwLock to provide thread-safe access to user data.
/// 
/// # Indexes
/// Maintains both primary (user ID) and secondary (email, phone) indexes
/// to enable efficient and flexible data retrieval.
/// 
/// # Serialization
/// Derives Default for easy initialization.
/// Skips serializing the internal Users collection to prevent complex serialization.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Memory {
    /// Internal users collection, not serialized
    #[serde(skip)]
    users: Users
}


/// Database Operations for Different Entity Types
/// 
/// This section provides a centralized implementation of CRUD operations 
/// for various domain entities using the in-memory database strategy.
/// 
/// ## Supported Entity Types
/// - Users
/// - Organisations
/// - Services
/// - Members
/// - Roles
/// - Verifications
/// - Resources
/// - Scopes

/// # User-related Database Operations
impl CreateItem<User> for Memory {
    type Error = Error;
    /// Creates a new user in the in-memory database
    /// 
    /// # Errors
    /// - Returns an error if a user with the same contact info already exists
    async fn create_item(&self, user: User) -> Result<User, Self::Error> {
        self.users.create_item(user).await
    }
}

impl GetItem<User> for Memory {
    type Error = Error;
    /// Retrieves a user by primary key (ID) or secondary key (email/phone)
    /// 
    /// # Returns
    /// - `Some(User)` if found
    /// - `None` if no matching user exists
    async fn get_item(&self, key: Key<&<User as Item>::PK, &<User as Item>::SK>) -> Result<Option<User>, Self::Error> {
        self.users.get_item(key).await
    }
}

impl UpdateItem<User> for Memory {
    type Error = Error;
    /// Updates an existing user's information
    /// 
    /// # Behavior
    /// - Allows full replacement of user data
    /// - Maintains index consistency
    async fn update_item(&self, key: Key<&<User as Item>::PK, &<User as Item>::SK>, user: User) -> Result<User, Self::Error> {
        self.users.update_item(key, user).await
    }

    /// Partially updates a user's information
    /// 
    /// # Supported Partial Updates
    /// - Username
    /// - First name
    /// - Last name
    /// - Password
    /// - Contact information
    async fn patch_item(&self, key: Key<&<User as Item>::PK, &<User as Item>::SK>, map: HashMap<String, Value>) -> Result<User, Self::Error> {
        self.users.patch_item(key, map).await
    }
}

impl DeleteItem<User> for Memory {
    type Error = Error;
    /// Deletes a user from the database
    /// 
    /// # Behavior
    /// - Removes user from primary and secondary indexes
    async fn delete_item(&self, key: Key<&<User as Item>::PK, &<User as Item>::SK>) -> Result<(), Self::Error> {
        self.users.delete_item(key).await
    }
}

// Similar placeholder implementations for other types would follow:
// - Organisation
// - Service
// - Member
// - Role
// - Verification
// - Resource
// - Scope
