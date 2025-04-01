use crate::domain::types::{User, OAuthClient, LoginMethod, Contact, Id, Error, Value};
use std::collections::HashMap;
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
     async fn authenticate(&self, provider: &str, code: &str) -> Result<User, Self::Error>;
 }




impl OAuth for OAuthClient {
    type Error = Error;

    async fn authorization_url(&self, provider: &str, redirect_url: &Url) -> Result<Url, Self::Error> {
        let provider_config = self.provider(provider).ok_or(Error::SocialProviderNotFound {provider: provider.to_string()})?;

        let mut auth_url = provider_config.auth_url.clone();
        auth_url.query_pairs_mut()
            .append_pair("client_id", &provider_config.client_id)
            .append_pair("response_type", "code")
            .append_pair("scope", &provider_config.scopes.join(" "))
            .append_pair("redirect_uri", redirect_url.as_str());

        Ok(auth_url)
    }

    async fn authenticate(&self, provider_name: &str, code: &str) -> Result<User, Self::Error> {
        let provider = self.provider(provider_name).ok_or(Error::SocialProviderNotFound {provider: provider_name.to_string()})?;

        // Exchange authorization code for access token
        let token_response = self.post(provider.token_url.as_str())
            .form(&[
                ("client_id", provider.client_id.as_str()),
                ("client_secret", provider.client_secret.as_str()),
                ("code", code),
                ("grant_type", "authorization_code")
            ])
            .send()
            .await
            .map_err(|e| Error::internal(e))?
            .json::<serde_json::Value>()
            .await
            .map_err(|e| Error::internal(e))?;

        let access_token = token_response["access_token"]
            .as_str()
            .ok_or(Error::IncorrectCode)?
            .to_string();

        // Retrieve user information
        let map = self.get(provider.userinfo_url.clone())
            .bearer_auth(&access_token)
            .send()
            .await
            .map_err(|e| Error::internal(e))?
            .json::<HashMap<String, Value>>()
            .await
            .map_err(|e| Error::internal(e))?;
        let fields =&provider.fields;

        println!("{:#?}", map);

        // Map user fields based on provider configuration
        User::from_provider(map, fields, provider_name)
    }
}
