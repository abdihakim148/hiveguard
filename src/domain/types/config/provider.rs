use crate::ports::outputs::oauth::{OAuth, TokenResponse};
use serde::{Serialize, Deserialize};
use super::{Github, super::Error};
use url::Url;


#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Provider {
    github: Github
}


impl Provider {
    pub fn authorization_url(&self, provider: &str, redirect_url: &Url) -> Result<Url, Error> {
        match provider {
            "github" => Ok(self.github.authorization_url(redirect_url)),
            _ => Err(Error::SocialProviderNotFound {provider: provider.into()})
        }
    }

    pub async fn authorize(&self, provider: &str, code: &str, redirect_url: &Url) -> Result<TokenResponse, Error> {
        match provider {
            "github" => Ok(self.github.authorize(code, redirect_url).await.map_err(Error::new)?),
            _ => Err(Error::SocialProviderNotFound {provider: provider.into()})?
        }
    }
}