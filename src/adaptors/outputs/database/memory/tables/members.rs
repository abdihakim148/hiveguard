use crate::domain::types::{Either, Key, Value, Member, Error, Id};
use crate::ports::outputs::database::{Table, Item};
use std::collections::{HashMap, HashSet};
use std::sync::RwLock;

/// The `Members` struct represents a table of members with primary and secondary indices.
/// The primary index is a map from the member's secondary key (SK) to the member itself.
/// The secondary index is a map from the member's primary key (PK) to a set of secondary keys (SK).
#[derive(Default, Debug)]
pub struct Members {
    data: RwLock<HashMap<(<Member as Item>::PK, <Member as Item>::SK), Member>>,
    primary: RwLock<HashMap<<Member as Item>::PK, HashSet<<Member as Item>::SK>>>,
    secondary: RwLock<HashMap<<Member as Item>::SK, HashSet<<Member as Item>::PK>>>,
}


impl Table for Members {
    type Error = Error;
    type Item = Member;
    type Map = HashMap<String, Value>;
    const NAME: &'static str = "Members";

    /// Creates a new instance of the `Members` table.
    ///
    /// # Returns
    ///
    /// * `Result<Self, Self::Error>` - A new instance of `Members` or an error.
    async fn new() -> Result<Self, Self::Error> {
        Ok(Members::default())
    }

    /// Inserts a new member into the table.
    ///
    /// # Arguments
    ///
    /// * `item` - A reference to the member to be inserted.
    ///
    /// # Returns
    ///
    /// * `Result<<Self::Item as Item>::PK, Self::Error>` - The primary key of the inserted member or an error.
    ///
    /// In this context, PK is the `org_id` and SK is the `user_id`.
    async fn create(&self, item: &Self::Item) -> Result<<Self::Item as Item>::PK, Self::Error> {
        let (pk, sk) = (&item.org_id, &item.user_id);
        let (mut data, mut primary, mut secondary) = (self.data.write()?, self.primary.write()?, self.secondary.write()?);

        if let Some(_) = data.get(&(pk.clone(), sk.clone())) {
            return Err(Error::Conflict(Self::Item::NAME));
        }

        data.insert((pk.clone(), sk.clone()), item.clone());
        primary.entry(pk.clone()).or_insert_with(HashSet::new).insert(sk.clone());
        secondary.entry(sk.clone()).or_insert_with(HashSet::new).insert(pk.clone());
        Ok(item.org_id.clone())
    }

    /// Retrieves a member from the table using a key.
    ///
    /// # Arguments
    ///
    /// * `key` - A key that can be either a primary key or a combination of primary and secondary keys.
    ///
    /// # Returns
    ///
    /// * `Result<Option<Self::Item>, Self::Error>` - The member if found, or None if not found.
    ///
    /// The SK is not used here because the SK represents the `user_id`, and a user can have
    /// multiple roles within an organisation. The function is designed to retrieve a single member,
    /// which is uniquely identified by the primary key (`org_id`).
    async fn get(&self, key: &Key<<Self::Item as Item>::PK, <Self::Item as Item>::SK>) -> Result<Option<Self::Item>, Self::Error> {
        let ck = match key {
            Key::Pk(pk) => return Ok(None),
            Key::Sk(sk) => return Ok(None),
            Key::Both((pk, sk)) => (*pk, *sk),
        };
        Ok(self.data.read()?.get(&ck).cloned())
    }

    /// Retrieves all organizations where a user is a member.
    ///
    /// # Arguments
    ///
    /// * `key` - An `Either` type that can be a primary key (org_id) or a secondary key (user_id).
    ///
    /// # Returns
    ///
    /// * `Result<Option<Vec<Self::Item>>, Self::Error>` - A vector of members if found, or None if not found.
    ///
    /// In this context, the method uses the `user_id` to find all associated organizations.
    ///
    /// # Arguments
    ///
    /// * `key` - An `Either` type that can be a primary key or a secondary key.
    ///
    /// # Returns
    ///
    /// * `Result<Option<Vec<Self::Item>>, Self::Error>` - A vector of members if found, or None if not found.
    ///
    /// In this context, PK is the `org_id` and SK is not used.
    async fn get_many(&self, key: Either<&<Self::Item as Item>::PK, &<Self::Item as Item>::SK>) -> Result<Option<Vec<Self::Item>>, Self::Error> {
        let data = self.data.read()?;
        let mut items = Vec::new();

        match key {
            Either::Left(pk) => {
                // Retrieve all members associated with the given org_id
                if let Some(sks) = self.primary.read()?.get(pk) {
                    for sk in sks {
                        if let Some(member) = data.get(&(*pk, *sk)) {
                            items.push(member.clone());
                        }
                    }
                }
            }
            Either::Right(sk) => {
                // Retrieve all members associated with the given user_id
                if let Some(pks) = self.secondary.read()?.get(sk) {
                    for pk in pks {
                        if let Some(member) = data.get(&(*pk, *sk)) {
                            items.push(member.clone());
                        }
                    }
                }
            }
        }

        if items.is_empty() {
            Ok(None)
        } else {
            Ok(Some(items))
        }
    }

    /// Updates specific fields of a member identified by its primary key.
    ///
    /// # Arguments
    ///
    /// * `id` - The primary key of the member to be updated.
    /// * `map` - A map containing the fields to be updated.
    ///
    /// # Returns
    ///
    /// * `Result<Self::Item, Self::Error>` - The updated member or an error.
    async fn patch(&self, key: &Key<<Self::Item as Item>::PK, <Self::Item as Item>::SK>, mut map: Self::Map) -> Result<Self::Item, Self::Error> {
        if let Some(member) = self.get(key).await? {
            let org_id = member.org_id;
            let title = map.remove("title").and_then(|v| v.try_into().ok()).unwrap_or(member.title.clone());
            let owner = map.remove("owner").and_then(|v| v.try_into().ok()).unwrap_or(member.owner);
            let roles = map.remove("roles").and_then(|v| v.try_into().ok()).unwrap_or(member.roles.clone());

            let updated_member = Member {
                org_id,
                user_id: member.user_id.clone(),
                title,
                owner,
                roles,
            };

            self.update(&updated_member).await?;
            return Ok(updated_member);
        }
        Err(Error::NotFound(Self::Item::NAME))
    }

    /// Updates a member in the table.
    ///
    /// # Arguments
    ///
    /// * `item` - A reference to the member to be updated.
    ///
    /// # Returns
    ///
    /// * `Result<(), Self::Error>` - An empty result or an error.
    async fn update(&self, item: &Self::Item) -> Result<(), Self::Error> {
        let (mut data, mut primary, mut secondary) = (self.data.write()?, self.primary.write()?, self.secondary.write()?);
        let pk = item.org_id.clone();
        let sk = item.user_id.clone();

        data.insert((pk.clone(), sk.clone()), item.clone());
        primary.entry(pk.clone()).or_insert_with(HashSet::new).insert(sk.clone());
        secondary.entry(sk.clone()).or_insert_with(HashSet::new).insert(pk.clone());
        Ok(())
    }

    /// Deletes a member from the table using its primary key.
    ///
    /// # Arguments
    ///
    /// * `id` - The primary key of the member to be deleted.
    ///
    /// # Returns
    ///
    /// * `Result<(), Self::Error>` - An empty result or an error.
    async fn delete(&self, key: &Key<<Self::Item as Item>::PK, <Self::Item as Item>::SK>) -> Result<(), Self::Error> {
        let key = match key {
            Key::Both((pk, sk)) => (*pk, *sk),
            _ => return Ok(())
        };
        let (mut data, mut primary, mut secondary) = (self.data.write()?, self.primary.write()?, self.secondary.write()?);
        if let Some(member) = data.remove(&key) {
            if let Some(set) = primary.get_mut(&member.org_id) {
                set.remove(&member.user_id);
                if set.is_empty() {
                    primary.remove(&member.org_id);
                }
            }
            if let Some(set) = secondary.get_mut(&member.user_id) {
                set.remove(&member.org_id);
                if set.is_empty() {
                    secondary.remove(&member.user_id);
                }
            }
            Ok(())
        } else {
            Err(Error::NotFound(Self::Item::NAME))
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::types::{Id, Member};
    use std::collections::HashMap;

    #[tokio::test]
    async fn test_create_member() {
        let members = Members::new().await.unwrap();
        let member = Member {
            org_id: Id::default(),
            user_id: Id::default(),
            title: "Developer".to_string(),
            owner: false,
            roles: vec![Id::default()],
        };

        let result = members.create(&member).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_member() {
        let members = Members::new().await.unwrap();
        let member = Member {
            org_id: Id::default(),
            user_id: Id::default(),
            title: "Developer".to_string(),
            owner: false,
            roles: vec![Id::default()],
        };

        members.create(&member).await.unwrap();
        let result = members.get(&Key::Both((member.org_id, member.user_id))).await.unwrap();
        assert_eq!(result, Some(member));
    }

    #[tokio::test]
    async fn test_get_many_members_by_org_id() {
        let members = Members::new().await.unwrap();
        let org_id = Id::default();
        let member1 = Member {
            org_id,
            user_id: Id::default(),
            title: "Developer".to_string(),
            owner: false,
            roles: vec![Id::default()],
        };
        let member2 = Member {
            org_id,
            user_id: Id::default(),
            title: "Manager".to_string(),
            owner: true,
            roles: vec![Id::default()],
        };

        members.create(&member1).await.unwrap();
        members.create(&member2).await.unwrap();
        let result = members.get_many(Either::Left(&org_id)).await.unwrap();
        assert_eq!(result.unwrap().len(), 2);
    }

    #[tokio::test]
    async fn test_get_many_members_by_user_id() {
        let members = Members::new().await.unwrap();
        let user_id = Id::default();
        let member1 = Member {
            org_id: Id::default(),
            user_id,
            title: "Developer".to_string(),
            owner: false,
            roles: vec![Id::default()],
        };
        let member2 = Member {
            org_id: Id::default(),
            user_id,
            title: "Manager".to_string(),
            owner: true,
            roles: vec![Id::default()],
        };

        members.create(&member1).await.unwrap();
        members.create(&member2).await.unwrap();
        let result = members.get_many(Either::Right(&user_id)).await.unwrap();
        assert_eq!(result.unwrap().len(), 2);
    }

    #[tokio::test]
    async fn test_update_member() {
        let members = Members::new().await.unwrap();
        let mut member = Member {
            org_id: Id::default(),
            user_id: Id::default(),
            title: "Developer".to_string(),
            owner: false,
            roles: vec![Id::default()],
        };

        members.create(&member).await.unwrap();
        member.title = "Senior Developer".to_string();
        members.update(&member).await.unwrap();
        let result = members.get(&Key::Both((member.org_id, member.user_id))).await.unwrap();
        assert_eq!(result.unwrap().title, "Senior Developer");
    }

    #[tokio::test]
    async fn test_delete_member() {
        let members = Members::new().await.unwrap();
        let member = Member {
            org_id: Id::default(),
            user_id: Id::default(),
            title: "Developer".to_string(),
            owner: false,
            roles: vec![Id::default()],
        };

        members.create(&member).await.unwrap();
        members.delete(&Key::Both((member.org_id, member.user_id))).await.unwrap();
        let result = members.get(&Key::Both((member.org_id, member.user_id))).await.unwrap();
        assert!(result.is_none());
    }
}
