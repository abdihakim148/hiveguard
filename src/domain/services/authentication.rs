use crate::ports::{Error, outputs::database::{Item, CreateItem, GetItem}};
use super::super::types::{Token, User, Paseto, Key, Audience};
use argon2::{PasswordHasher, PasswordVerifier};
use super::Password;


pub trait Authentication: Sized + Item {
    type Error;
    type QueryKey;
    async fn register<DB: CreateItem<Self>, H: PasswordHasher>(self, db: &DB, hasher: &H) -> Result<Self, Self::Error>;
    async fn authenticate<DB: GetItem<Self>, V: PasswordVerifier>(query_key: &Self::QueryKey, password: &str, db: &DB, verifier: &V, paseto: &Paseto, issuer: String, audience: Audience) -> Result<Token, Self::Error>;
}



impl Authentication for User {
    type Error = Error;
    type QueryKey = Self::SK;

    async fn register<DB: CreateItem<Self>, H: PasswordHasher>(mut self, db: &DB, hasher: &H) -> Result<Self, Self::Error> {
        self.password = self.password.hash(hasher)?;
        let mut user = db.create_item(self).await?;
        user.password = Default::default();
        Ok(user)
    }


    async fn authenticate<DB: GetItem<Self>, V: PasswordVerifier>(contact: &Self::QueryKey, password: &str, db: &DB, verifier: &V, paseto: &Paseto, issuer: String, audience: Audience) -> Result<Token, Self::Error> {
        let key = Key::Sk(contact);
        let user = db.get_item(key).await?;
        let hash = &user.password;
        password.verify(hash, verifier)?;
        let keys = &paseto.keys;
        let ttl = paseto.ttl;
        let token = user.token(issuer, audience, ttl);
        Ok(token)
    }
}