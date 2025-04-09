use reqwest::{Client, RequestBuilder, Method};
use serde::{Deserialize, Serialize};
use crate::domain::types::Error;
use std::collections::HashMap;
use crate::ports::ErrorTrait;
use static_init::dynamic;
use chrono::Duration;
use url::Url;

#[dynamic]
pub static CLIENT: Client = Client::new();

/// Trait defining OAuth authentication flow
pub trait OAuth {
    fn client_id(&self) -> &str;
    fn client_secret(&self) -> &str;
    fn auth_url(&self) -> &Url;
    fn token_url(&self) -> &Url;
    fn scope(&self) -> Option<&str>;

    fn client(&self) -> &Client {
        &CLIENT
    }

    /// Generate authorization URL for initiating OAuth flow
    fn authorization_url(&self, redirect_url: &Url) -> Url {
        let mut url = self.auth_url().clone();
        url.query_pairs_mut().append_pair("client_id", self.client_id());
        url.query_pairs_mut().append_pair("redirect_uri", redirect_url.as_str());
        url.query_pairs_mut().append_pair("response_type", "code");
        url.query_pairs_mut().append_pair("access_type", "offline");
        if let Some(scope) = self.scope() {
            url.query_pairs_mut().append_pair("scope", scope);
        }
        url
    }

    /// exchange code with a token
    async fn authorize(&self, code: &str, redirect_url: &Url) -> Result<TokenResponse, impl ErrorTrait + Send + Sync> {
        let client = self.client();
        let form: HashMap<&str, &str> = [
            ("grant_type", "authorization_code"),
            ("code", code),
            ("redirect_uri", redirect_url.as_str()),
            ("client_id", self.client_id()),
            ("client_secret", self.client_secret())
        ].into();
        let res = client.post(self.token_url().clone())
        .header("Accept", "application/json")
        .form(&form).send().await.map_err(Error::internal)?
        .json().await.map_err(Error::internal)?;
        Ok::<_, Error>(res)
    }
}


#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TokenResponse {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: Option<Duration>,
    pub scope: Option<String>,
    pub id_token: Option<String>,
    pub refresh_token: Option<String>
}