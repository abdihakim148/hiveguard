use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
use crate::ports::outputs::oauth::BasicClient;
use crate::impl_oauth_provider_serde;
use serde::de::{Visitor, MapAccess};
use serde::ser::{SerializeStruct};
use oauth2::{Scope, ClientId, ClientSecret, AuthUrl, TokenUrl, basic};
use std::fmt;
use url::Url;


#[derive(Debug, Clone)]
pub struct Github {
   pub client: BasicClient,
   pub client_secret: String,
   pub scopes: Vec<Scope>,
}


impl Github {
    const CLIENT_ID: &'static str = "$CLIENT_ID";
    const CLIENT_SECRET: &'static str = "$CLIENT_SECRET";
    const AUTH_URL: &'static str = "https://github.com/login/oauth/authorize";
    const TOKEN_URL: &'static str = "https://github.com/login/oauth/access_token";
    const SCOPES: [&'static str; 2] = ["user:email", "user:phone"];
}


impl_oauth_provider_serde!(Github, "Github");


impl Default for Github {
    fn default() -> Self {
        let client_id = ClientId::new(String::from(Self::CLIENT_ID));
        let client_secret = ClientSecret::new(String::from(Self::CLIENT_SECRET));
        let auth_url = AuthUrl::new(String::from(Self::AUTH_URL)).expect("invalid Auth_url for github default");
        let token_url = TokenUrl::new(String::from(Self::TOKEN_URL)).expect("invalid token_url for github default");
        let scopes = Self::SCOPES.into_iter().map(|scope|{Scope::new(String::from(scope))}).collect::<Vec<_>>();
        let client = basic::BasicClient::new(client_id).set_auth_uri(auth_url).set_client_secret(client_secret.clone()).set_token_uri(token_url);
        let client_secret = client_secret.into_secret();
        Github{client, client_secret, scopes}
    }
}