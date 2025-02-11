//! Members collection implementation for the memory database
//! 
//! This module provides the implementation for storing and managing member records
//! in memory with thread-safe access and index management.

use crate::ports::outputs::database::{Item, CreateItem, GetItem, UpdateItem, DeleteItem};
use crate::domain::types::{Member, Id, Key, Value};
use super::error::Error;
use std::collections::HashMap;
use std::sync::RwLock as Lock;

/// Thread-safe, indexed storage for member records
/// 
/// # Indexes
/// - Primary index: (Organisation ID, User ID) -> Member record
/// - Secondary indexes:
///   * Organisation ID -> Vec<Member>
///   * User ID -> Vec<Member>
/// 
/// # Concurrency
/// Uses RwLock to ensure safe concurrent read and write operations
#[derive(Debug, Default)]
pub struct Members {
    /// Primary storage of members, keyed by (org_id, user_id)
    pub members: Lock<HashMap<(<Member as Item>::PK, <Member as Item>::SK), Member>>,
    
    /// Secondary index mapping organisation IDs to user IDs
    pub org_index: Lock<HashMap<<Member as Item>::PK, Vec<<Member as Item>::SK>>>,
    
    /// Secondary index mapping user IDs to organisation IDs
    pub user_index: Lock<HashMap<<Member as Item>::SK, Vec<<Member as Item>::PK>>>,
}

impl Members {
    /// Updates secondary indexes when a member's details change
    /// 
    /// # Arguments
    /// * `member`: The member being added or updated
    /// 
    /// # Behavior
    /// - Adds/updates member in organisation and user indexes
    pub fn update_indexes(&self, member: &Member) -> Result<(), Error> {
        // Update organisation index
        let mut org_index = self.org_index.write()?;
        org_index.entry(member.org_id)
            .or_insert_with(Vec::new)
            .retain(|&user_id| user_id != member.user_id);
        org_index.get_mut(&member.org_id)
            .unwrap()
            .push(member.user_id);

        // Update user index
        let mut user_index = self.user_index.write()?;
        user_index.entry(member.user_id)
            .or_insert_with(Vec::new)
            .retain(|&org_id| org_id != member.org_id);
        user_index.get_mut(&member.user_id)
            .unwrap()
            .push(member.org_id);

        Ok(())
    }

    /// Removes a member from secondary indexes
    /// 
    /// # Arguments
    /// * `member`: The member being removed
    pub fn remove_from_indexes(&self, member: &Member) -> Result<(), Error> {
        // Remove from organisation index
        let mut org_index = self.org_index.write()?;
        if let Some(org_members) = org_index.get_mut(&member.org_id) {
            org_members.retain(|&user_id| user_id != member.user_id);
        }

        // Remove from user index
        let mut user_index = self.user_index.write()?;
        if let Some(user_members) = user_index.get_mut(&member.user_id) {
            user_members.retain(|&org_id| org_id != member.org_id);
        }

        Ok(())
    }
}

impl CreateItem<Member> for Members {
    type Error = Error;
    
    async fn create_item(&self, member: Member) -> Result<Member, Self::Error> {
        // Check if member already exists
        if self.members.read()?.contains_key(&(member.org_id, member.user_id)) {
            return Err(Error::MemberAlreadyExists);
        }
        
        // Update indexes
        self.update_indexes(&member)?;
        
        // Store member
        self.members.write()?.insert((member.org_id, member.user_id), member.clone());
        
        Ok(member)
    }
}

impl GetItem<Member> for Members {
    type Error = Error;
    
    async fn get_item(&self, key: Key<&<Member as Item>::PK, &<Member as Item>::SK>) -> Result<Option<Member>, Self::Error> {
        match key {
            Key::Pk(pk) => {
                // Get first user for an organisation
                let user_id = self.org_index.read()?
                    .get(pk)
                    .and_then(|user_ids| user_ids.first().cloned());
                
                match user_id {
                    Some(user_id) => {
                        Ok(self.members.read()?.get(&(*pk, user_id)).cloned())
                    },
                    None => Ok(None)
                }
            },
            Key::Sk(sk) => {
                // Get first organisation for a user
                let org_id = self.user_index.read()?
                    .get(sk)
                    .and_then(|org_ids| org_ids.first().cloned());
                
                match org_id {
                    Some(org_id) => {
                        Ok(self.members.read()?.get(&(org_id, *sk)).cloned())
                    },
                    None => Ok(None)
                }
            },
            Key::Both((org_id, user_id)) => {
                // Get specific member by organisation and user ID
                Ok(self.members.read()?.get(&(*org_id, *user_id)).cloned())
            }
        }
    }
}

impl UpdateItem<Member> for Members {
    type Error = Error;

    async fn update_item(&self, _: Key<&<Member as Item>::PK, &<Member as Item>::SK>, member: Member) -> Result<Member, Self::Error> {
        // Remove old indexes
        if let Some(old_member) = self.members.read()?.get(&(member.org_id, member.user_id)) {
            self.remove_from_indexes(old_member)?;
        }
        
        // Update indexes
        self.update_indexes(&member)?;
        
        // Store updated member
        self.members.write()?.insert((member.org_id, member.user_id), member.clone());
        Ok(member)
    }

    async fn patch_item(&self, key: Key<&<Member as Item>::PK, &<Member as Item>::SK>, mut map: HashMap<String, Value>) -> Result<Member, Self::Error> {
        let (org_id, user_id) = match key {
            Key::Both((org_id, user_id)) => (*org_id, *user_id),
            Key::Pk(org_id) => return Err(Error::MemberNotFound),
            Key::Sk(user_id) => return Err(Error::MemberNotFound),
        };

        let mut members = self.members.write()?;
        let member = members.get_mut(&(org_id, user_id)).ok_or(Error::MemberNotFound)?;
        
        // Update basic fields
        if let Some(value) = map.remove("title") {
            member.title = value.try_into()?;
        }
        if let Some(value) = map.remove("owner") {
            member.owner = value.try_into()?;
        }
        if let Some(value) = map.remove("roles") {
            member.roles = value.try_into()?;
        }

        // Remove old indexes and update with new member data
        self.remove_from_indexes(member)?;
        self.update_indexes(member)?;

        Ok(member.clone())
    }
}

impl DeleteItem<Member> for Members {
    type Error = Error;
    
    async fn delete_item(&self, key: Key<&<Member as Item>::PK, &<Member as Item>::SK>) -> Result<(), Self::Error> {
        let (org_id, user_id) = match key {
            Key::Both((org_id, user_id)) => (*org_id, *user_id),
            Key::Pk(org_id) => return Err(Error::MemberNotFound),
            Key::Sk(user_id) => return Err(Error::MemberNotFound),
        };

        // Remove from indexes
        if let Some(member) = self.members.read()?.get(&(org_id, user_id)) {
            self.remove_from_indexes(member)?;
        }

        // Remove member
        self.members.write()?.remove(&(org_id, user_id));
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bson::oid::ObjectId;

    /// Helper function to create a test member
    fn create_test_member() -> Member {
        Member {
            org_id: Id(ObjectId::new()),
            user_id: Id(ObjectId::new()),
            title: "Test Member".to_string(),
            owner: false,
            roles: vec![Id(ObjectId::new())],
        }
    }

    #[tokio::test]
    async fn test_create_member() {
        let members = Members::default();
        let member = create_test_member();
        let result = members.create_item(member.clone()).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), member);
    }

    #[tokio::test]
    async fn test_create_duplicate_member() {
        let members = Members::default();
        let member1 = create_test_member();
        let mut member2 = member1.clone();
        
        let _ = members.create_item(member1).await;
        let result = members.create_item(member2).await;
        assert!(matches!(result, Err(Error::MemberAlreadyExists)));
    }

    #[tokio::test]
    async fn test_get_member_by_org() {
        let members = Members::default();
        let member = create_test_member();
        let _ = members.create_item(member.clone()).await;
        
        let result = members.get_item(Key::Pk(&member.org_id)).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Some(member));
    }

    #[tokio::test]
    async fn test_get_member_by_user() {
        let members = Members::default();
        let member = create_test_member();
        let _ = members.create_item(member.clone()).await;

        let result = members.get_item(Key::Sk(&member.user_id)).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Some(member));
    }

    #[tokio::test]
    async fn test_update_member() {
        let members = Members::default();
        let mut member = create_test_member();
        let _ = members.create_item(member.clone()).await;
        
        member.title = "Updated Member".to_string();
        let result = members.update_item(Key::Both((&member.org_id, &member.user_id)), member.clone()).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), member);
    }

    #[tokio::test]
    async fn test_delete_member() {
        let members = Members::default();
        let member = create_test_member();
        let _ = members.create_item(member.clone()).await;
        
        let result = members.delete_item(Key::Both((&member.org_id, &member.user_id))).await;
        assert!(result.is_ok());
        
        let get_result = members.get_item(Key::Both((&member.org_id, &member.user_id))).await;
        assert_eq!(get_result.unwrap(), None);
    }
}
