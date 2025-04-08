use crate::domain::types::OAuthClient; // Use re-exported path
use crate::domain::types::Error;
use crate::ports::outputs::oauth::OAuth;
use crate::ports::ErrorTrait; // Required by the OAuth trait definition
use oauth2::{AuthorizationCode, CsrfToken, TokenResponse};
use reqwest::Client; // Needed for request_async
use url::Url;

impl OAuth for OAuthClient {
    type Error = Error; // Assuming crate::domain::types::Error implements ErrorTrait

    async fn authorization_url(&self, provider: &str, _redirect_url: &Url) -> Result<Url, Self::Error> {
        // _redirect_url is unused in the original impl, marked as such.
        let client = self.client(); // Assuming self.client() returns reqwest::Client
        let provider_config = self.provider(provider).ok_or(Error::SocialProviderNotFound{provider: provider.into()})?;
        let scopes = provider_config.scopes.clone();
        let basic_client = &provider_config.client;
        let (url, _) = basic_client
            .authorize_url(CsrfToken::new_random)
            .add_scopes(scopes)
            .url();
        Ok(url)
    }

    async fn authenticate(
        &self,
        provider: &str,
        code: &str,
    ) -> Result<impl TokenResponse, Self::Error> {
        let provider_config = self.provider(provider).ok_or(Error::SocialProviderNotFound{provider: provider.into()})?;
        let client = self.client(); // Assuming self.client() returns reqwest::Client
        let basic_client = &provider_config.client;

        let token_result = basic_client
            .exchange_code(AuthorizationCode::new(code.to_string()))
            .request_async(client) // Pass the reqwest client here
            .await
            .map_err(|e| Error::internal(e))?; // Assuming Error::internal exists

        Ok(token_result)
    }
}