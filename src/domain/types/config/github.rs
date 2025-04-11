use super::super::{Error, User, Value, EmailAddress, Contact, LoginMethod};
use crate::domain::services::oauth::client::{OAuth, OpenId, TokenResponse};
use actix_web::http::header::LastModified;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use url::Url;


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Github {
   pub client_id: String,
   pub client_secret: String,
   pub auth_url: Url,
   pub token_url: Url,
   pub user_info_url: Url,
   pub user_emails_url: Url,
   pub scope: Option<String>,
}


impl Github {
    const NAME: &'static str = "github";
    const CLIENT_ID: &'static str = "$CLIENT_ID";
    const CLIENT_SECRET: &'static str = "$CLIENT_SECRET";
    const AUTH_URL: &'static str = "https://github.com/login/oauth/authorize";
    const TOKEN_URL: &'static str = "https://github.com/login/oauth/access_token";
    const USER_INFO_URL: &'static str = "https://api.github.com/user";
    const USER_EMAILS_URL: &'static str = "https://api.github.com/user/emails";
    const SCOPE: &str = "user:email";
}


impl Default for Github {
    fn default() -> Self {
        let client_id = String::from(Self::CLIENT_ID);
        let client_secret = String::from(Self::CLIENT_SECRET);
        let auth_url = Url::parse(Self::AUTH_URL).expect("invalid Auth_url for github default");
        let token_url = Url::parse(Self::TOKEN_URL).expect("invalid token_url for github default");
        let user_info_url = Url::parse(Self::USER_INFO_URL).expect("invalid user_info_url for github default");
        let user_emails_url = Url::parse(Self::USER_EMAILS_URL).expect("invalid user_emails_url for github default");
        let scope = Some(Self::SCOPE.into());
        Github{client_id, client_secret, auth_url, token_url, user_info_url, user_emails_url, scope}
    }
}



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


impl Github {
    async fn user(&self, token: &str) -> Result<User, Error> {
        let email = self.email(token).await?;
        let mut map = self.info(token).await?;
        let id = Default::default();
        let username = map.remove("login").ok_or(Error::could_not_get_necessary_info("could not get user's username from github"))?.try_into().map_err(Error::could_not_get_necessary_info)?;
        let name = map.remove("name").ok_or(Error::could_not_get_necessary_info("could not get user's name from github"))?.try_into().map_err(Error::could_not_get_necessary_info)?;
        let contact = Contact::Email(email);
        let login = LoginMethod::Social(String::from(Self::NAME));
        let password = Default::default();
        let profile = map.remove("avatar_url").unwrap_or_default().option().map_err(Error::could_not_get_necessary_info)?;
        Ok(User{id, username, name, contact, login, password, profile})
    }

    async fn info(&self, token: &str) -> Result<HashMap<String, Value>, Error> {
        let client = self.client();
        let url = self.user_info_url.clone();
        let res = client.get(url).bearer_auth(token).send().await.map_err(Error::could_not_get_necessary_info)?;
        res.json::<HashMap<String, Value>>().await.map_err(Error::could_not_get_necessary_info)
    }

    async fn email(&self, token: &str) -> Result<EmailAddress, Error> {
        let client = self.client();
        let url = self.user_emails_url.clone();
        let res = client.get(url).bearer_auth(token).send().await.map_err(Error::could_not_get_email)?;
        let results = res.json::<Vec<HashMap<String, Value>>>().await.map_err(Error::could_not_get_email)?;
        let mut email = None;
        for mut map in results {
            let primary: bool = map.remove("primary").unwrap_or_default().try_into()?;
            let verified: bool = map.remove("verified").unwrap_or_default().try_into()?;
            let email_str: String = map.remove("email").unwrap_or_default().try_into()?;
            email = Some(EmailAddress::new(&email_str, verified).map_err(Error::could_not_get_email)?);
            /// break the loop if it is the primary email.
            if primary {
                break
            }
        }
        Ok(email.ok_or(Error::could_not_get_email("could not get any valid email address from github"))?)
    }
}


impl OpenId for Github {
    async fn authenticate(&self, code: &str, redirect_url: &Url) -> Result<(User, TokenResponse), Error> {
        let token_response = self.authorize(code, redirect_url).await?;
        let token = &token_response.access_token;
        let user = self.user(token).await?;
        Ok((user, token_response))
    }
}