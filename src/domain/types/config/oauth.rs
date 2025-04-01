use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use super::Provider;
use reqwest::Client;
use std::ops::Deref;


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
}


impl Deref for OAuthClient {
    type Target = Client;

    fn deref(&self) -> &Self::Target {
        &self.client
    }
}