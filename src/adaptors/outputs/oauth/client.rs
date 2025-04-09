use crate::ports::outputs::oauth::OAuth;
use crate::domain::types::Github;
use url::Url;


impl OAuth for Github {
    fn client_id(&self) -> &str {
        &self.client_id
    }

    fn client_secret(&self) -> &str {
        &self.client_secret
    }

    fn auth_url(&self) -> &Url {
        &self.auth_url
    }

    fn token_url(&self) -> &Url {
        &self.token_url
    }

    fn scope(&self) -> Option<&str> {
        match &self.scope {
            Some(scope) => Some(scope),
            _ => None
        }
    }
}