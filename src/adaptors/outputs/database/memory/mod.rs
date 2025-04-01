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

mod organisations;
mod error;
mod users;
mod members;
mod services;
mod verifications;

use crate::ports::outputs::database::{Item, CreateItem, GetItem, GetItems, UpdateItem, DeleteItem, Map};
use crate::domain::types::{User, Key, Value, Organisation, Member, Service, Verification, Id, LoginMethod};
use std::collections::{HashMap, HashSet};
use serde::{Serialize, Deserialize};
use std::sync::RwLock as Lock;
use organisations::*;
use std::hash::Hash;
use std::fmt::Debug;
use services::*;
use members::*;
use error::*;
use users::*;
use verifications::*;

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
pub struct Memory<ID = Id> 
where
    ID: Clone + Hash + PartialEq + Eq + Debug + Send + Sync + 'static
{
    /// Internal users collection, not serialized
    #[serde(skip)]
    users: Users,
    
    /// Internal organisations collection, not serialized
    #[serde(skip)]
    organisations: Organisations,
    
    /// Internal members collection, not serialized
    #[serde(skip)]
    members: Members,
    
    /// Internal services collection, not serialized
    #[serde(skip)]
    services: Services,
    
    /// Internal verifications collection, not serialized
    #[serde(skip)]
    verifications: Verifications<ID>
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
impl<ID> CreateItem<User> for Memory<ID> 
where
    ID: Clone + Hash + PartialEq + Eq + Debug + Send + Sync + 'static
{
    type Error = Error;
    /// Creates a new user in the in-memory database
    /// 
    /// # Errors
    /// - Returns an error if a user with the same contact info already exists
    async fn create_item(&self, user: User) -> Result<User, Self::Error> {
        self.users.create_item(user).await
    }
}

impl<ID> GetItem<User> for Memory<ID> 
where
    ID: Clone + Hash + PartialEq + Eq + Debug + Send + Sync + 'static
{
    type Error = Error;
    /// Retrieves a user by primary key (ID) or secondary key (email/phone)
    /// 
    /// # Returns
    /// - `Some(User)` if found
    /// - `None` if no matching user exists
    async fn get_item(&self, key: Key<&<User as Item>::PK, &<User as Item>::SK>) -> Result<User, Self::Error> {
        self.users.get_item(key).await
    }
}

impl<ID> UpdateItem<User> for Memory<ID> 
where
    ID: Clone + Hash + PartialEq + Eq + Debug + Send + Sync + 'static
{
    type Error = Error;
    type Update = Map;
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
    async fn patch_item(&self, key: Key<&<User as Item>::PK, &<User as Item>::SK>, update: Map) -> Result<User, Self::Error> {
        self.users.patch_item(key, update).await
    }

    async fn delete_fields(&self, key: Key<&<User as Item>::PK, &<User as Item>::SK>, fields: HashSet<String>) -> Result<User, Self::Error> {
        self.users.delete_fields(key, fields).await
    }
}

impl<ID> DeleteItem<User> for Memory<ID> 
where
    ID: Clone + Hash + PartialEq + Eq + Debug + Send + Sync + 'static
{
    type Error = Error;
    /// Deletes a user from the database
    /// 
    /// # Behavior
    /// - Removes user from primary and secondary indexes
    async fn delete_item(&self, key: Key<&<User as Item>::PK, &<User as Item>::SK>) -> Result<(), Self::Error> {
        self.users.delete_item(key).await
    }
}


/// # Organisation-related Database Operations
impl<ID> CreateItem<Organisation> for Memory<ID> 
where
    ID: Clone + Hash + PartialEq + Eq + Debug + Send + Sync + 'static
{
    type Error = Error;
    /// Creates a new organisation in the in-memory database
    /// 
    /// # Errors
    /// - Returns an error if an organisation with the same name already exists
    async fn create_item(&self, organisation: Organisation) -> Result<Organisation, Self::Error> {
        self.organisations.create_item(organisation).await
    }
}

impl<ID> GetItem<Organisation> for Memory<ID> 
where
    ID: Clone + Hash + PartialEq + Eq + Debug + Send + Sync + 'static
{
    type Error = Error;
    /// Retrieves an organisation by primary key (ID) or secondary key (name)
    /// 
    /// # Returns
    /// - `Some(Organisation)` if found
    /// - `None` if no matching organisation exists
    async fn get_item(&self, key: Key<&<Organisation as Item>::PK, &<Organisation as Item>::SK>) -> Result<Organisation, Self::Error> {
        self.organisations.get_item(key).await
    }
}

impl<ID> UpdateItem<Organisation> for Memory<ID> 
where
    ID: Clone + Hash + PartialEq + Eq + Debug + Send + Sync + 'static
{
    type Error = Error;
    type Update = Map;
    /// Updates an existing organisation's information
    /// 
    /// # Behavior
    /// - Allows full replacement of organisation data
    /// - Maintains index consistency
    async fn update_item(&self, key: Key<&<Organisation as Item>::PK, &<Organisation as Item>::SK>, organisation: Organisation) -> Result<Organisation, Self::Error> {
        self.organisations.update_item(key, organisation).await
    }

    /// Partially updates an organisation's information
    /// 
    /// # Supported Partial Updates
    /// - Name
    async fn patch_item(&self, key: Key<&<Organisation as Item>::PK, &<Organisation as Item>::SK>, update: Map) -> Result<Organisation, Self::Error> {
        self.organisations.patch_item(key, update).await
    }

    async fn delete_fields(&self, key: Key<&<Organisation as Item>::PK, &<Organisation as Item>::SK>, fields: HashSet<String>) -> Result<Organisation, Self::Error> {
        self.organisations.delete_fields(key, fields).await
    }
}

impl<ID> DeleteItem<Organisation> for Memory<ID> 
where
    ID: Clone + Hash + PartialEq + Eq + Debug + Send + Sync + 'static
{
    type Error = Error;
    /// Deletes an organisation from the database
    /// 
    /// # Behavior
    /// - Removes organisation from primary and secondary indexes
    async fn delete_item(&self, key: Key<&<Organisation as Item>::PK, &<Organisation as Item>::SK>) -> Result<(), Self::Error> {
        self.organisations.delete_item(key).await
    }
}

/// # Member-related Database Operations
impl<ID> CreateItem<Member> for Memory<ID> 
where
    ID: Clone + Hash + PartialEq + Eq + Debug + Send + Sync + 'static
{
    type Error = Error;
    /// Creates a new member in the in-memory database
    /// 
    /// # Errors
    /// - Returns an error if the member already exists in the organisation
    async fn create_item(&self, member: Member) -> Result<Member, Self::Error> {
        self.members.create_item(member).await
    }
}

impl<ID> GetItem<(Organisation, User), Member> for Memory<ID> 
where
    ID: Clone + Hash + PartialEq + Eq + Debug + Send + Sync + 'static
{
    type Error = Error;
    /// Retrieves a member by organisation ID, user ID, or both
    /// 
    /// # Returns
    /// - `Some(Member)` if found
    /// - `None` if no matching member exists
    async fn get_item(&self, key: Key<&<(Organisation, User) as Item>::PK, &<(Organisation, User) as Item>::SK>) -> Result<Member, Self::Error> {
        self.members.get_item(key).await
    }
}

impl<ID> UpdateItem<(Organisation, User), Member> for Memory<ID> 
where
    ID: Clone + Hash + PartialEq + Eq + Debug + Send + Sync + 'static
{
    type Error = Error;
    type Update = Map;
    /// Updates an existing member's information
    /// 
    /// # Behavior
    /// - Allows full replacement of member data
    /// - Maintains index consistency
    async fn update_item(&self, key: Key<&<(Organisation, User) as Item>::PK, &<(Organisation, User) as Item>::SK>, member: Member) -> Result<Member, Self::Error> {
        self.members.update_item(key, member).await
    }

    /// Partially updates a member's information
    /// 
    /// # Supported Partial Updates
    /// - Title
    /// - Owner status
    /// - Roles
    async fn patch_item(&self, key: Key<&<(Organisation, User) as Item>::PK, &<(Organisation, User) as Item>::SK>, update: Map) -> Result<Member, Self::Error> {
        self.members.patch_item(key, update).await
    }

    async fn delete_fields(&self, key: Key<&<(Organisation, User) as Item>::PK, &<(Organisation, User) as Item>::SK>, fields: HashSet<String>) -> Result<Member, Self::Error> {
        self.members.delete_fields(key, fields).await
    }
}

impl<ID> DeleteItem<Member> for Memory<ID> 
where
    ID: Clone + Hash + PartialEq + Eq + Debug + Send + Sync + 'static
{
    type Error = Error;
    /// Deletes a member from the database
    /// 
    /// # Behavior
    /// - Removes member from primary and secondary indexes
    async fn delete_item(&self, key: Key<&<Member as Item>::PK, &<Member as Item>::SK>) -> Result<(), Self::Error> {
        self.members.delete_item(key).await
    }
}

/// # Service-related Database Operations
impl<ID> CreateItem<Service> for Memory<ID> 
where
    ID: Clone + Hash + PartialEq + Eq + Debug + Send + Sync + 'static
{
    type Error = Error;
    /// Creates a new service in the in-memory database
    /// 
    /// # Errors
    /// - Returns an error if a service with the same name already exists
    async fn create_item(&self, service: Service) -> Result<Service, Self::Error> {
        self.services.create_item(service).await
    }
}

impl<ID> GetItem<Service> for Memory<ID> 
where
    ID: Clone + Hash + PartialEq + Eq + Debug + Send + Sync + 'static
{
    type Error = Error;
    /// Retrieves a service by primary key (ID) or secondary key (owner ID)
    /// 
    /// # Returns
    /// - `Some(Service)` if found
    /// - `None` if no matching service exists
    async fn get_item(&self, key: Key<&<Service as Item>::PK, &<Service as Item>::SK>) -> Result<Service, Self::Error> {
        self.services.get_item(key).await
    }
}

impl<ID> UpdateItem<Service> for Memory<ID> 
where
    ID: Clone + Hash + PartialEq + Eq + Debug + Send + Sync + 'static
{
    type Error = Error;
    type Update = Map;
    /// Updates an existing service's information
    /// 
    /// # Behavior
    /// - Allows full replacement of service data
    /// - Maintains index consistency
    async fn update_item(&self, key: Key<&<Service as Item>::PK, &<Service as Item>::SK>, service: Service) -> Result<Service, Self::Error> {
        self.services.update_item(key, service).await
    }

    /// Partially updates a service's information
    /// 
    /// # Supported Partial Updates
    /// - Name
    /// - Client secret
    /// - Redirect URIs
    /// - Scopes
    /// - Grant types
    /// - Token expiry
    /// - Owner ID
    async fn patch_item(&self, key: Key<&<Service as Item>::PK, &<Service as Item>::SK>, update: Map) -> Result<Service, Self::Error> {
        self.services.patch_item(key, update).await
    }

    async fn delete_fields(&self, key: Key<&<Service as Item>::PK, &<Service as Item>::SK>, fields: HashSet<String>) -> Result<Service, Self::Error> {
        self.services.delete_fields(key, fields).await
    }
}

impl<ID> DeleteItem<Service> for Memory<ID> 
where
    ID: Clone + Hash + PartialEq + Eq + Debug + Send + Sync + 'static
{
    type Error = Error;
    /// Deletes a service from the database
    /// 
    /// # Behavior
    /// - Removes service from primary and secondary indexes
    async fn delete_item(&self, key: Key<&<Service as Item>::PK, &<Service as Item>::SK>) -> Result<(), Self::Error> {
        self.services.delete_item(key).await
    }
}

/// User-related GetItems Operations
impl<ID> GetItems<User, Organisation> for Memory<ID> 
where
    ID: Clone + Hash + PartialEq + Eq + Debug + Send + Sync + 'static
{
    type Error = Error;
    type Filter = bool;

    async fn get_items(&self, key: Key<&<User as Item>::PK, &<User as Item>::SK>, filter: Self::Filter) -> Result<Vec<Organisation>, Self::Error> {
        // Find the user ID based on the key
        let user_id = match key {
            Key::Pk(pk) => *pk,
            Key::Sk(sk) => self.users.pk(sk)?.ok_or(Error::UserNotFound)?,
            Key::Both((pk, _)) => *pk
        };

        // Collect organisation IDs and member information in a single read lock
        let (org_ids, members_map) = {
            
            let org_ids = self.members.user_index.read()?.get(&user_id).cloned().unwrap_or_default();
            
            let mut members_map = HashMap::new();
            for &org_id in &org_ids {
                if let Some(member) = self.members.members.read()?.get(&(org_id, user_id)) {
                    members_map.insert(org_id, member.clone());
                }
            }
            
            (org_ids, members_map)
        };

        // Retrieve organisations with minimal lock time
        let mut organisations = Vec::new();
        
        for &org_id in &org_ids {
            if let Some(org) = self.organisations.organisations.read()?.get(&org_id) {
                if !filter || members_map.get(&org_id).map_or(false, |m| m.owner) {
                    organisations.push(org.clone());
                }
            }
        }

        Ok(organisations)
    }
}

/// Organisation-related GetItems Operations
impl<ID> GetItems<Organisation, User> for Memory<ID> 
where
    ID: Clone + Hash + PartialEq + Eq + Debug + Send + Sync + 'static
{
    type Error = Error;
    type Filter = bool;

    async fn get_items(&self, key: Key<&<Organisation as Item>::PK, &<Organisation as Item>::SK>, filter: Self::Filter) -> Result<Vec<User>, Self::Error> {
        // Find the organisation ID based on the key
        let org_id = match key {
            Key::Pk(pk) => *pk,
            Key::Sk(sk) => match self.organisations.pk(sk)? {
                Some(pk) => pk,
                None => return Err(Error::OrganisationNotFound)
            },
            Key::Both((pk, _)) => *pk
        };

        // Use the Members table to find users for this organisation
        let user_ids = self.members.org_index.read()?
            .get(&org_id)
            .cloned()
            .unwrap_or_default();

        // Retrieve users, optionally filtering by ownership
        let mut result = Vec::new();
        for user_id in user_ids {
            if let Some(user) = self.users.users.read()?.get(&user_id) {
                // If filter is true, only return users who are owners of the organisation
                if let Some(member) = self.members.members.read()?.get(&(org_id, user_id)) {
                    if !filter || member.owner {
                        result.push(user.clone());
                    }
                }
            }
        }

        Ok(result)
    }
}

impl<ID> GetItems<Organisation, (Member, User)> for Memory<ID> 
where
    ID: Clone + Hash + PartialEq + Eq + Debug + Send + Sync + 'static
{
    type Error = Error;
    type Filter = bool;

    async fn get_items(&self, key: Key<&<Organisation as Item>::PK, &<Organisation as Item>::SK>, filter: Self::Filter) -> Result<Vec<(Member, User)>, Self::Error> {
        // Find the organisation ID based on the key
        let org_id = match key {
            Key::Pk(pk) => *pk,
            Key::Sk(sk) => match self.organisations.pk(sk)? {
                Some(pk) => pk,
                None => return Err(Error::OrganisationNotFound)
            },
            Key::Both((pk, _)) => *pk
        };

        // Use the Members table to find users for this organisation
        let user_ids = self.members.org_index.read()?
            .get(&org_id)
            .cloned()
            .unwrap_or_default();

        // Retrieve users and members, optionally filtering by ownership
        let mut result = Vec::new();
        for user_id in user_ids {
            if let (Some(user), Some(member)) = (
                self.users.users.read()?.get(&user_id).cloned(), 
                self.members.members.read()?.get(&(org_id, user_id)).cloned()
            ) {
                // If filter is true, only return users who are owners of the organisation
                if !filter || member.owner {
                    result.push((member, user));
                }
            }
        }

        Ok(result)
    }
}

impl<ID> GetItems<User, (Member, Organisation)> for Memory<ID> 
where
    ID: Clone + Hash + PartialEq + Eq + Debug + Send + Sync + 'static
{
    type Error = Error;
    type Filter = bool;

    async fn get_items(&self, key: Key<&<User as Item>::PK, &<User as Item>::SK>, filter: Self::Filter) -> Result<Vec<(Member, Organisation)>, Self::Error> {
        // Find the user ID based on the key
        let user_id = match key {
            Key::Pk(pk) => *pk,
            Key::Sk(sk) => match self.users.pk(sk)? {
                Some(pk) => pk,
                None => return Err(Error::UserNotFound)
            },
            Key::Both((pk, _)) => *pk
        };

        // Use the Members table to find organisations for this user
        let org_ids = self.members.user_index.read()?
            .get(&user_id)
            .cloned()
            .unwrap_or_default();

        // Retrieve organisations and members, optionally filtering by ownership
        let mut result = Vec::new();
        for org_id in org_ids {
            if let (Some(organisation), Some(member)) = (
                self.organisations.organisations.read()?.get(&org_id).cloned(),
                self.members.members.read()?.get(&(org_id, user_id)).cloned()
            ) {
                // If filter is true, only return organisations where the user is an owner
                if !filter || member.owner {
                    result.push((member, organisation));
                }
            }
        }

        Ok(result)
    }
}

/// # Verification-related Database Operations
impl<ID> CreateItem<Verification<ID>> for Memory<ID>
where
    ID: Clone + Hash + PartialEq + Eq + Debug + Send + Sync + 'static,
    Verifications<ID>: CreateItem<Verification<ID>, Error = Error>
{
    type Error = Error;
    /// Creates a new verification in the in-memory database
    async fn create_item(&self, verification: Verification<ID>) -> Result<Verification<ID>, Self::Error> {
        self.verifications.create_item(verification).await
    }
}

impl<ID> GetItem<Verification<ID>> for Memory<ID> 
where
    ID: Clone + Hash + PartialEq + Eq + Debug + Send + Sync + 'static,
    Verifications<ID>: GetItem<Verification<ID>, Error = Error>
{
    type Error = Error;
    /// Retrieves a verification by primary key (contact) or secondary key (ID)
    /// 
    /// # Returns
    /// - The verification if found
    /// - Error if no matching verification exists
    async fn get_item(&self, key: Key<&<Verification<ID> as Item>::PK, &<Verification<ID> as Item>::SK>) -> Result<Verification<ID>, Self::Error> {
        self.verifications.get_item(key).await
    }
}

// Similar placeholder implementations for other types would follow:
// - Role
// - Resource
// - Scope

#[cfg(test)]
mod tests {
    use crate::domain::types::{Id, User, Organisation, Member, Contact, EmailAddress, Phone};
    use bson::oid::ObjectId;
    use super::*;

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
            ),
            login: LoginMethod::Password
        }
    }

    fn create_test_organisation() -> Organisation {
        Organisation {
            id: Id(ObjectId::new()),
            name: "Test Organisation".to_string(),
            domain: None,
            home: None,
            contacts: Default::default(),
        }
    }

    fn create_test_member(org: &Organisation, user: &User) -> Member {
        Member {
            org_id: org.id,
            user_id: user.id,
            title: "Test Member".to_string(),
            owner: false,
            roles: vec![Id(ObjectId::new())],
        }
    }

    #[tokio::test]
    async fn test_get_user_organisations() {
        let db = Memory::<Id>::default();
        let user = create_test_user();
        let org = create_test_organisation();
        let member = create_test_member(&org, &user);

        // Create test data
        db.create_item(user.clone()).await.unwrap();
        db.create_item(org.clone()).await.unwrap();
        db.create_item(member).await.unwrap();

        // Test getting all organisations
        let orgs: Vec<Organisation> = GetItems::<User, Organisation>::get_items(&db, Key::Pk(&user.id), false).await.unwrap();
        assert!(!orgs.is_empty());
        assert_eq!(orgs.len(), 1);

        // Test getting owned organisations only (should be empty as member.owner = false)
        let owned_orgs: Vec<Organisation> = GetItems::<User, Organisation>::get_items(&db, Key::Pk(&user.id), true).await.unwrap();
        assert!(owned_orgs.is_empty());
    }

    #[tokio::test]
    async fn test_get_organisation_users() {
        let db = Memory::<Id>::default();
        let user = create_test_user();
        let org = create_test_organisation();
        let member = create_test_member(&org, &user);

        // Create test data
        db.create_item(user.clone()).await.unwrap();
        db.create_item(org.clone()).await.unwrap();
        db.create_item(member).await.unwrap();

        // Test getting all users
        let users: Vec<User> = GetItems::<Organisation, User>::get_items(&db, Key::Pk(&org.id), false).await.unwrap();
        assert!(!users.is_empty());
        assert_eq!(users.len(), 1);

        // Test getting owners only (should be empty as member.owner = false)
        let owners: Vec<User> = GetItems::<Organisation, User>::get_items(&db, Key::Pk(&org.id), true).await.unwrap();
        assert!(owners.is_empty());
    }

    #[tokio::test]
    async fn test_get_organisation_members_and_users() {
        let db = Memory::<Id>::default();
        let user = create_test_user();
        let org = create_test_organisation();
        let member = create_test_member(&org, &user);

        // Create test data
        db.create_item(user.clone()).await.unwrap();
        db.create_item(org.clone()).await.unwrap();
        db.create_item(member.clone()).await.unwrap();

        // Test getting all members and users
        let members: Vec<(Member, User)> = GetItems::<Organisation, (Member, User)>::get_items(&db, Key::Pk(&org.id), false).await.unwrap();
        assert!(!members.is_empty());
        assert_eq!(members.len(), 1);
        assert_eq!(members[0].0, member);
        assert_eq!(members[0].1, user);

        // Test getting owner members only (should be empty as member.owner = false)
        let owners: Vec<(Member, User)> = GetItems::<Organisation, (Member, User)>::get_items(&db, Key::Pk(&org.id), true).await.unwrap();
        assert!(owners.is_empty());
    }

    #[tokio::test]
    async fn test_get_user_members_and_organisations() {
        let db = Memory::<Id>::default();
        let user = create_test_user();
        let org = create_test_organisation();
        let member = create_test_member(&org, &user);

        // Create test data
        db.create_item(user.clone()).await.unwrap();
        db.create_item(org.clone()).await.unwrap();
        db.create_item(member.clone()).await.unwrap();

        // Test getting all members and organisations
        let members: Vec<(Member, Organisation)> = GetItems::<User, (Member, Organisation)>::get_items(&db, Key::Pk(&user.id), false).await.unwrap();
        assert!(!members.is_empty());
        assert_eq!(members.len(), 1);
        assert_eq!(members[0].0, member);
        assert_eq!(members[0].1, org);

        // Test getting owner memberships only (should be empty as member.owner = false)
        let owners: Vec<(Member, Organisation)> = GetItems::<User, (Member, Organisation)>::get_items(&db, Key::Pk(&user.id), true).await.unwrap();
        assert!(owners.is_empty());
    }
}
