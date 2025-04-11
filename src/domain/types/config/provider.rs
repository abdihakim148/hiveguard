use crate::domain::types::{User, Error, Token, Contact, Key, Audience, Paseto};
use crate::domain::services::oauth::client::{OAuth, OpenId, TokenResponse};
use crate::ports::outputs::database::{CreateItem, GetItem};
use crate::ports::outputs::verify::Verifyer;
use serde::{Serialize, Deserialize};
use super::Github;
use url::Url;


#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Provider {
    github: Github
}


impl Provider {
    pub fn authorization_url(&self, provider: &str, redirect_url: &Url) -> Result<Url, Error> {
        match provider {
            "github" => Ok(self.github.authorization_url(redirect_url)),
            _ => Err(Error::SocialProviderNotFound {provider: provider.into()})
        }
    }

    pub async fn authorize(&self, provider: &str, code: &str, redirect_url: &Url) -> Result<(User, TokenResponse), Error> {
        match provider {
            "github" => Ok(self.github.authenticate(code, redirect_url).await.map_err(Error::new)?),
            _ => Err(Error::SocialProviderNotFound {provider: provider.into()})?
        }
    }

    #[cfg(feature = "email")]
    pub async fn authenticate<DB: CreateItem<User> + GetItem<User> + CreateItem<V::Verification>, V: Verifyer>(&self, provider: &str, redirect_url: &Url, db: &DB, verifyer: &V, code: &str, base_url: &str, channel: V::Channel, paseto: &Paseto, issuer: String, audience: Audience) -> Result<Option<(User, Token)>, Error> {
        let (user, _) = self.authorize(provider, code, redirect_url).await?;
        if !user.contact.verified()? {
            let contact = match &user.contact {
                Contact::Email(email) => email,
                Contact::Both(_, email) => email,
                Contact::Phone(_) => return Err(Error::could_not_get_email(format!("got phone number instead of email from {provider}")))
            };
            verifyer.initiate(contact, channel, base_url, db).await.map_err(Error::new)?;
            return Ok(None)
        }
        let key = Key::Sk(&user.contact);
        let existing_user = db.get_item(key).await.map_err(Error::new)?;
        let user = match existing_user {
            Some(user) => user,
            None => db.create_item(user).await.map_err(Error::new)?
        };

        let keys = &paseto.keys;
        let ttl = paseto.ttl;
        let token = user.token(issuer, audience, ttl);
        Ok(Some((user, token)))
    }
}