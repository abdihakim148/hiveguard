use super::super::types::{Token, User, Paseto, Key, Audience, Error as DomainError, Id, LoginMethod, Error};
use crate::ports::{outputs::database::{CreateItem, GetItem, Item, UpdateItem}};
use super::{Password, Paseto as PasetoTrait, Verification};
use argon2::{PasswordHasher, PasswordVerifier};
use crate::ports::outputs::verify::Verifyer;


pub trait Authentication: Sized + Item {
    type QueryKey;
    type Error;
    async fn register<DB: CreateItem<Self> + GetItem<Self> + CreateItem<V::Verification>, H: PasswordHasher, V: Verifyer>(
        self, 
        db: &DB, 
        hasher: &H,
        channel: V::Channel,
        base_url: &str,
        verifyer: &V
    ) -> Result<(), Self::Error>;
    
    async fn authenticate<DB: GetItem<Self> + CreateItem<V::Verification> + UpdateItem<Self>, H: PasswordVerifier, V: Verifyer>(
        query_key: &Self::QueryKey, 
        password: &str, 
        db: &DB, 
        hasher: &H,
        paseto: &Paseto,
        issuer: String, 
        audience: Audience,
        channel: V::Channel,
        base_url: &str,
        verifyer: &V
    ) -> Result<Option<(Self, Token)>, Self::Error>;
    
    async fn authorize(
        token: &str, 
        paseto: &Paseto
    ) -> Result<<Self as Item>::PK, Self::Error>;
}



impl Authentication for User {
    type QueryKey = Self::SK;
    type Error = Error;

    async fn register<DB: CreateItem<Self> + GetItem<Self> + CreateItem<V::Verification>, H: PasswordHasher, V: Verifyer>(
        mut self,
        db: &DB,
        hasher: &H,
        channel: V::Channel,
        base_url: &str,
        verifyer: &V
    ) -> Result<(), Self::Error> {
        // make sure the user has provided the required contact before any operations.
        let contact = self.contact.clone().contact()?;

        self.password = self.password.hash(hasher)?;
        let user = db.create_item(self).await.map_err(Error::new)?;


        // send verification code to the user.
        verifyer.initiate_verification(contact, channel, base_url, db).await.map_err(Error::new)
    }

    async fn authenticate<DB: GetItem<Self> + CreateItem<V::Verification> + UpdateItem<Self>, H: PasswordVerifier, V: Verifyer>(
        contact: &Self::QueryKey, 
        password: &str, 
        db: &DB, 
        hasher: &H, 
        paseto: &Paseto,
        issuer: String, 
        audience: Audience,
        channel: V::Channel,
        base_url: &str,
        verifyer: &V
    ) -> Result<Option<(Self, Token)>, Self::Error> {
        let key = Key::Sk(contact);
        let mut user = db.get_item(key).await.map_err(Error::new)?;
        
        let contact = user.contact.clone().contact()?;
        
        if !contact.verified() {
            verifyer.initiate_verification(contact, channel, base_url, db).await.map_err(Error::new)?;
            return Ok(None)
        }

        if user.login != LoginMethod::Password {
            return Err(Error::IncorrectLoginMethod)
        }

        // verify that the provided password matches the stored password 
        password.verify(&user.password, hasher)?;

        // Generate token with verification status
        let keys = &paseto.keys;
        let ttl = paseto.ttl;
        let token = user.token(issuer, audience, ttl);
        user.password = Default::default();
        Ok(Some((user, token)))
    }

    async fn authorize(signature: &str, paseto: &Paseto) -> Result<Id, Self::Error> {
        let keys = &paseto.keys;
        let token = Token::try_verify(signature, keys)?;
        
        if token.expired() {
            Err(DomainError::TokenExpired)?
        }
        
        Ok(token.subject)
    }
}
