use super::oauth::{OAuth, TokenResponse};
use crate::domain::types::{User, Error};
use url::Url;

pub trait OpenId: OAuth {
    async fn authenticate(&self, code: &str, redirect_url: &Url) -> Result<(User, TokenResponse), Error>;
}