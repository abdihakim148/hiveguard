use serde::{Serialize, Deserialize};
use super::{Provider, BasicClient};
use std::collections::HashMap;
use reqwest::Client;
use std::ops::Deref;
use oauth2::Scope;

type Credentials = (BasicClient, Vec<Scope>);


#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct OAuthClient {
    #[serde(flatten)]
    providers: HashMap<String, Provider>,
    #[serde(skip)]
    client: Client
}

impl OAuthClient {
    pub fn provider(&self, name: &str) -> Option<&Provider> {
        self.providers.get(name)
    }

    pub fn client(&self) -> &Client {
        &self.client
    }
}


impl Deref for OAuthClient {
    type Target = Client;

    fn deref(&self) -> &Self::Target {
        &self.client
    }
}