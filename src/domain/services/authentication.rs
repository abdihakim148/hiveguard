use async_trait::async_trait;
use crate::domain::types::{Credentials, SessionToken, Result};

#[async_trait]
pub trait Authentication {
    type Id;
    type Details;

    async fn register<E: Entity>(&self, entity: &E) -> Result<Self::Id>;
    async fn authenticate<E: Entity>(&self, credentials: &Credentials) -> Result<SessionToken>;
    async fn deauthenticate(&self, token: &SessionToken) -> Result<()>;
    async fn verify(&self, token: &SessionToken) -> Result<bool>;
    async fn update_credentials<E: Entity>(&self, entity_id: &E::Id, new_credentials: &Credentials) -> Result<()>;
    async fn reset_credentials(&self, identifier: &str) -> Result<()>;
    async fn get_entity_details<E: Entity>(&self, token: &SessionToken) -> Result<E::Details>;
}
