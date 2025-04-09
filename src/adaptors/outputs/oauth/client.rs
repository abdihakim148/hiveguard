use crate::domain::types::Github; // Use re-exported path
use crate::domain::types::Error;
use crate::ports::outputs::oauth::OAuth;
use crate::ports::ErrorTrait; // Required by the OAuth trait definition
use oauth2::{AuthorizationCode, CsrfToken, TokenResponse, RedirectUrl};
use reqwest::Client; // Needed for request_async
use url::Url;
use std::borrow::Cow;

impl OAuth for Github {
    fn oauth_client(&self) -> &crate::ports::outputs::oauth::BasicClient {
        &self.client
    }

    fn scopes(&self) -> Vec<oauth2::Scope> {
        self.scopes.clone()
    }
}