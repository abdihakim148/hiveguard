#![allow(unused)]
use crate::domain::types::Result;

/**
 * A trait representing authentication operations.
 *
 * This trait provides methods for registering, authenticating, deauthenticating,
 * verifying, updating credentials, resetting credentials, and retrieving entity details.
 */
pub trait Authentication: Sized {
    type Id;
    type Value;
    type Token;

    async fn register(&self) -> Result<Self::Id>;
    async fn authenticate(&self, value: &Self::Value) -> Result<Self::Token>;
    async fn deauthenticate(&self, token: &Self::Token) -> Result<()>;
    async fn verify(&self, token: &Self::Token) -> Result<bool>;
    async fn update_credentials(&self, entity_id: &Self::Id, new_value: &Self::Value) -> Result<()>;
    async fn reset_credentials(&self, identifier: &str) -> Result<()>;
    async fn get_entity_details(&self, token: &Self::Token) -> Result<Self>;
}
