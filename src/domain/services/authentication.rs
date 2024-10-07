use async_trait::async_trait;
use crate::domain::types::Result;

#[async_trait]
pub trait Authentication {
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
