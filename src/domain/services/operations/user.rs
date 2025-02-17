use crate::ports::{Error, outputs::database::GetItem};
use crate::domain::types::{User, Id, Key};
use super::Get;


impl Get for User {
    type Error = Error;
    type Filter = Id;

    async fn get<DB: GetItem<Self>>(id: &Self::Filter, db: &DB) -> Result<Self, Self::Error> {
        let key = Key::Pk(id);
        let mut user = db.get_item(key).await?;
        user.password = Default::default();
        Ok(user)
    }
}