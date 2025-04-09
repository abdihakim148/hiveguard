use oauth2::{basic, AuthUrl, ClientId, ClientSecret, EndpointNotSet, EndpointSet, Scope, TokenUrl, RedirectUrl, CsrfToken, AuthorizationCode, TokenResponse};
use crate::domain::types::Error;
use crate::ports::ErrorTrait;
use static_init::dynamic;
use std::borrow::Cow;
use url::Url;

pub type BasicClient<T1 = EndpointNotSet, T2 = EndpointNotSet, T3 = EndpointNotSet> = basic::BasicClient<EndpointSet, T1, T2, T3, EndpointSet>;
#[dynamic]
pub static CLIENT: reqwest::Client = reqwest::Client::new();

/// Trait defining OAuth authentication flow
pub trait OAuth {
    fn oauth_client(&self) -> &BasicClient;
    fn scopes(&self) -> Vec<Scope>;

    fn client(&self) -> &reqwest::Client {
        &CLIENT
    }

    /// Generate authorization URL for initiating OAuth flow
    fn authorization_url(&self, redirect_url: &RedirectUrl) -> Url {
        let redirect_url = Cow::Borrowed(redirect_url);
        let client = self.client();
        let scopes = self.scopes();
        let basic_client = self.oauth_client();
        let (url, _) = basic_client
            .authorize_url(CsrfToken::new_random)
            .add_scopes(scopes)
            .set_redirect_uri(redirect_url)
            .url();
        url
    }

    /// exchange code with a token
    async fn authorize(&self, code: String) -> Result<impl TokenResponse, impl ErrorTrait + Send + Sync> {
        let client = self.client();
        let basic_client = self.oauth_client();
        let token = basic_client
            .exchange_code(AuthorizationCode::new(code))
            .request_async(client) // Pass the reqwest client here
            .await
            .map_err(|e| Error::internal(e))?;

        Ok::<_, Error>(token)
    }
}
