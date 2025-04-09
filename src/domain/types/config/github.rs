use serde::{Serialize, Deserialize};
use url::Url;


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Github {
   pub client_id: String,
   pub client_secret: String,
   pub auth_url: Url,
   pub token_url: Url,
   pub scope: Option<String>,
}


impl Github {
    const CLIENT_ID: &'static str = "$CLIENT_ID";
    const CLIENT_SECRET: &'static str = "$CLIENT_SECRET";
    const AUTH_URL: &'static str = "https://github.com/login/oauth/authorize";
    const TOKEN_URL: &'static str = "https://github.com/login/oauth/access_token";
    const SCOPE: &str = "user:email user:phone";
}


impl Default for Github {
    fn default() -> Self {
        let client_id = String::from(Self::CLIENT_ID);
        let client_secret = String::from(Self::CLIENT_SECRET);
        let auth_url = Url::parse(Self::AUTH_URL).expect("invalid Auth_url for github default");
        let token_url = Url::parse(Self::TOKEN_URL).expect("invalid token_url for github default");
        let scope = Some(Self::SCOPE.into());
        Github{client_id, client_secret, auth_url, token_url, scope}
    }
}