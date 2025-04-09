use super::oauth::{OAuth, TokenResponse};
use crate::domain::types::User;
use crate::ports::ErrorTrait;

pub trait OpenId: OAuth {
    async fn auhtenticate(&self) -> Result<(User, TokenResponse), impl ErrorTrait + Send + Sync>;
}