use super::super::types::{Token, User, Paseto, Key, Audience, Error as DomainError, Id, LoginMethod, Error, Value, Either, Contact}; // Added Contact
use crate::ports::{outputs::database::{CreateItem, GetItem, Item, UpdateItem}, outputs::verify::Code};
use super::{Password, Paseto as PasetoTrait, Verification, Tokenizer};
use argon2::{PasswordHasher, PasswordVerifier};
use serde::{Serialize, de::DeserializeOwned};
use crate::ports::outputs::verify::Verifyer;
// Removed duplicate Tokenizer import: use crate::domain::services::Tokenizer;
use std::collections::HashMap;


pub trait Authentication: Sized + Item {
    type QueryKey;
    type Error: From<DomainError>; // Ensure DomainError can be converted
    type Token: Serialize + DeserializeOwned + Send + Sync + 'static;
    async fn register<DB: CreateItem<Self> + GetItem<Self> + CreateItem<V::Verification>, H: PasswordHasher, V: Verifyer>(
        self,
        db: &DB,
        hasher: &H,
        channel: V::Channel,
        base_url: &str,
        verifyer: &V
    ) -> Result<(), Self::Error>;

    async fn confirm_verification<Tok: Tokenizer<Self::Token>, DB: GetItem<V::Verification> + UpdateItem<Self, Update = HashMap<String, Value>>, V: Verification<E, Self, DIGITS>, E: Clone + Send + Sync, const DIGITS: usize>(
        tokenizer: &Tok,
        db: &DB,
        verifyer: &V,
        contact: E,
        code: Either<&str, &<V::Verification as Code<E, DIGITS>>::Id>,
        issuer: String,
        audience: Audience,
    ) -> Result<(Self, String), Self::Error>;

    async fn authenticate<T: Tokenizer<Self::Token>, DB: GetItem<Self> + CreateItem<V::Verification> + UpdateItem<Self>, H: PasswordVerifier, V: Verifyer>(
        query_key: &Self::QueryKey,
        password: &str,
        db: &DB,
        hasher: &H,
        tokenizer: &T,
        issuer: String,
        audience: Audience,
        channel: V::Channel,
        base_url: &str,
        verifyer: &V
    ) -> Result<Option<(Self, String)>, Self::Error>;

    async fn authorize<T: Tokenizer<Self::Token>>(
        token: &str,
        tokenizer: &T
    ) -> Result<<Self as Item>::PK, Self::Error>;
}



impl Authentication for User {
    type QueryKey = Contact; // Changed from Self::SK to Contact based on usage
    type Error = Error;
    type Token = Token;

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

        // send verification code to the user.
        // use the initiate method of the Verify trait to avoid checking is the user exists.
        // because user is not created at this time.
        verifyer.initiate(&contact, channel, base_url, db).await.map_err(Error::new)?;

        self.password = self.password.hash(hasher)?;

        let _ = db.create_item(self).await.map_err(Error::new)?;
        Ok(())
    }

    
    async fn confirm_verification<T: Tokenizer<Self::Token>, DB: GetItem<V::Verification> + UpdateItem<Self, Update = HashMap<String, Value>>, V: Verification<E, Self, DIGITS>, E: Clone + Send + Sync, const DIGITS: usize>(
        tokenizer: &T,
        db: &DB,
        verifyer: &V,
        contact: E,
        code: Either<&str, &<V::Verification as Code<E, DIGITS>>::Id>,
        issuer: String,
        audience: Audience,
    ) -> Result<(Self, String), Self::Error> {
        // Delegate verification confirmation to the Verification trait implementation
        // This will also update the user's verification status in the database
        let mut user = verifyer.confirm_verification(contact, code, db).await.map_err(Error::new)?; // Map error explicitly

        // Generate token for the verified user
        let input = (&user, audience, issuer);
        let token = tokenizer.try_sign(input)?;

        // Clear password before returning user data
        user.password = Default::default();

        Ok((user, token))
    }

    async fn authenticate<T: Tokenizer<Self::Token>, DB: GetItem<Self> + CreateItem<V::Verification> + UpdateItem<Self>, H: PasswordVerifier, V: Verifyer>(
        contact: &Self::QueryKey, // Changed from &Self::QueryKey to &Contact
        password: &str,
        db: &DB,
        hasher: &H,
        tokenizer: &T,
        issuer: String,
        audience: Audience,
        channel: V::Channel,
        base_url: &str,
        verifyer: &V
    ) -> Result<Option<(Self, String)>, Self::Error> {
        let key = Key::Sk(contact);
        let mut user = db.get_item(key).await.map_err(Error::new)?.ok_or(Error::item_not_found(User::NAME))?;

        let user_contact = user.contact.clone().contact()?; // Renamed variable to avoid conflict

        if !user_contact.verified() {
            verifyer.initiate_verification(user_contact, channel, base_url, db).await.map_err(Error::new)?;
            return Ok(None)
        }

        if user.login != LoginMethod::Password {
            return Err(Error::IncorrectLoginMethod)
        }

        // verify that the provided password matches the stored password
        password.verify(&user.password, hasher)?;

        // Generate token with verification status
        let input = (&user, audience, issuer);
        let token = tokenizer.try_sign(input)?;
        user.password = Default::default();
        Ok(Some((user, token)))
    }

    async fn authorize<T: Tokenizer<Self::Token>>(signature: &str, tokenizer: &T) -> Result<Id, Self::Error> {
        let token = tokenizer.try_verify(signature)?;

        if token.expired() {
            Err(DomainError::TokenExpired)? // Use From trait here
        }

        Ok(token.subject)
    }
}
