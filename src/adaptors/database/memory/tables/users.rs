use crate::ports::output::database::{Table, Result};
use crate::domain::types::User;
use std::collections::HashMap;
use bson::oid::ObjectId;
use std::sync::RwLock;


pub struct Users {
    emails: RwLock<HashMap<String, ObjectId>>,
    users: RwLock<HashMap<ObjectId, User>>,
}
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

    async fn create(&self, user: &Self::Item) -> Result<Self::Id> {
        let mut users = self.users.write().unwrap();
        let mut emails = self.emails.write().unwrap();

        if emails.contains_key(&user.email) {
            return Err(crate::domain::types::Error::InvalidInput("Email already exists".into()));
        }

        let id = ObjectId::new();
        users.insert(id.clone(), user.clone());
        emails.insert(user.email.clone(), id.clone());

        Ok(id)
    }

    async fn read(&self, id: &Self::Id) -> Option<Self::Item> {
        let users = self.users.read().unwrap();
        users.get(id).cloned()
    }

    async fn update(&self, user: &Self::Item) -> Result<Self::Id> {
        let mut users = self.users.write().unwrap();
        let mut emails = self.emails.write().unwrap();

        if let Some(existing_id) = emails.get(&user.email) {
            if existing_id != &user.id {
                return Err(crate::domain::types::Error::InvalidInput("Email already exists".into()));
            }
        }

        users.insert(user.id.clone(), user.clone());
        emails.insert(user.email.clone(), user.id.clone());

        Ok(user.id.clone())
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
