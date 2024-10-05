use crate::ports::output::database::{Table, Result};
use crate::domain::types::User;
use std::collections::HashMap;
use bson::oid::ObjectId;
use std::sync::{Arc, RwLock};


pub struct Users {
    emails: RwLock<HashMap<String, ObjectId>>,
    users: RwLock<HashMap<ObjectId, User>>,
}
#[async_trait::async_trait]
impl Table for Users {
    type Item = User;
    type Id = ObjectId;

    const NAME: &'static str = "users";

    async fn new() -> Result<Self> {
        Ok(Users {
            emails: RwLock::new(HashMap::new()),
            users: RwLock::new(HashMap::new()),
        })
    }

    async fn create(&self, item: &Self::Item) -> Result<Self::Id> {
        let mut users = self.users.write().unwrap();
        let mut emails = self.emails.write().unwrap();

        if emails.contains_key(&item.email) {
            return Err(crate::domain::types::Error::InvalidInput("Email already exists".into()));
        }

        let id = ObjectId::new();
        users.insert(id.clone(), item.clone());
        emails.insert(item.email.clone(), id.clone());

        Ok(id)
    }

    async fn read(&self, id: &Self::Id) -> Option<Self::Item> {
        let users = self.users.read().unwrap();
        users.get(id).cloned()
    }

    async fn update(&self, item: &Self::Item) -> Result<Self::Id> {
        let mut users = self.users.write().unwrap();
        let mut emails = self.emails.write().unwrap();

        if let Some(existing_id) = emails.get(&item.email) {
            if existing_id != &item.id {
                return Err(crate::domain::types::Error::InvalidInput("Email already exists".into()));
            }
        }

        users.insert(item.id.clone(), item.clone());
        emails.insert(item.email.clone(), item.id.clone());

        Ok(item.id.clone())
    }

    async fn delete(&self, id: &Self::Id) -> Result<Self::Id> {
        let mut users = self.users.write().unwrap();
        let mut emails = self.emails.write().unwrap();

        if let Some(user) = users.remove(id) {
            emails.remove(&user.email);
            Ok(id.clone())
        } else {
            Err(crate::domain::types::Error::NotFound)
        }
    }
}
