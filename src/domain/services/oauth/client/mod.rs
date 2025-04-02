use oauth2::{AuthorizationCode, TokenResponse, CsrfToken};
use crate::domain::types::{OAuthClient, Error};
use crate::ports::ErrorTrait;
use reqwest::Client;
use url::Url;

/// Trait defining OAuth authentication flow
pub trait OAuth {
    type Error: ErrorTrait;
    
    /// Generate authorization URL for initiating OAuth flow
    /// This method is publicly accessible
    async fn authorization_url(&self, provider: &str, redirect_url: &Url) -> Result<Url, Self::Error>;

    /// Authenticate user using provider and authorization code
    /// This method is publicly accessible
    async fn authenticate(&self, provider: &str, code: &str) -> Result<impl TokenResponse, Self::Error>;
}

impl OAuth for OAuthClient {
    type Error = Error;

    async fn authorization_url(&self, provider: &str, redirect_url: &Url) -> Result<Url, Self::Error> {
        let client = self.client();
        let provider = self.provider(provider).ok_or(Error::SocialProviderNotFound{provider: provider.into()})?;
        let scopes = provider.scopes.clone();
        let basic_client = &provider.client;
        let (url, _) = basic_client.authorize_url(CsrfToken::new_random)
            .add_scopes(scopes).url();
        Ok(url)
    }

    async fn authenticate(&self, provider: &str, code: &str) -> Result<impl TokenResponse, Self::Error> {
        let provider = self.provider(provider).ok_or(Error::SocialProviderNotFound{provider: provider.into()})?;
        let client = self.client();
        let basic_client = &provider.client;

        let token_result = basic_client
            .exchange_code(AuthorizationCode::new(code.to_string()))
            .request_async(client)
            .await
            .map_err(|e| Error::internal(e))?;

        Ok(token_result)
    }
}
