use crate::ports::outputs::database::{GetItem, Item, Map, UpdateItem};
use crate::domain::types::{User, Id, Key, Value, Error};
use std::collections::HashMap;
use super::{Get, Update};


impl Get for User {
    type Error = Error;
    type Filter = Id;

    async fn get<DB: GetItem<Self>>(id: &Self::Filter, db: &DB) -> Result<Self, Self::Error> {
        let key = Key::Pk(id);
        let mut user = db.get_item(key).await.map_err(Error::new)?.ok_or(Error::item_not_found(User::NAME))?;
        user.password = Default::default();
        Ok(user)
    }
}



impl Update for User {
    type Error = Error;
    type Filter = Id;

    async fn update<DB: UpdateItem<Self, Update = Map> + GetItem<Self>>(id: &Self::Filter, db: &DB, item: HashMap<String, Value>) -> Result<Self, Self::Error> {
        let mut update = HashMap::new();

        // Only allow updating specific fields
        if let Some(username) = item.get("username") {
            update.insert("username".to_string(), username.clone());
        }
        if let Some(first_name) = item.get("first_name") {
            update.insert("first_name".to_string(), first_name.clone());
        }
        if let Some(last_name) = item.get("last_name") {
            update.insert("last_name".to_string(), last_name.clone());
        }

        let key = Key::Pk(id);
        if update.is_empty() {
            return Ok(db.get_item(key).await.map_err(Error::new)?.ok_or(Error::item_not_found(User::NAME))?)
        }

        let mut user = db.patch_item(key, update).await.map_err(Error::new)?;
        user.password = Default::default();
        Ok(user)
    }
}
