use rusty_paseto::core::{V1, V2, V4, Public, Paseto as Builder, Payload, Footer};
use crate::domain::services::{Tokenizer, Result};
use serde::{Serialize, de::DeserializeOwned};
use crate::domain::types::Error;
use super::{Paseto, Version};



impl Paseto {
    fn try_sign(&self, payload: &str) -> Result<String> {
        let id = self.current.id.to_string();
        let id =  id.as_str();
        let footer = Footer::from(id);
        let mut key = [0u8; 64];
        key[..32].copy_from_slice(self.current.private_key.as_slice());
        key[32..].copy_from_slice(self.current.public_key.as_slice());
        let version = self.current.version;
        let payload = Payload::from(payload);
        match version {
            Version::V1 => {
                let key = From::from(key.as_slice());
                Ok(Builder::<V1, Public>::builder().set_payload(payload).set_footer(footer).try_sign(&key)?)
            },
            Version::V2 => {
                let key = rusty_paseto::core::Key::from(&key);
                let key = From::from(&key);
                Ok(Builder::<V2, Public>::builder().set_payload(payload).set_footer(footer).try_sign(&key)?)
            },
            Version::V4 => {
                let key = rusty_paseto::core::Key::from(&key);
                let key = From::from(&key);
                Ok(Builder::<V4, Public>::builder().set_payload(payload).set_footer(footer).try_sign(&key)?)
            }
        }
    }

    fn try_verify<Payload: Serialize + DeserializeOwned + Send + Sync + 'static>(id: &str, public_key: [u8; 32], version: Version, token: &str) -> Result<Payload> {
        let footer = Some(Footer::from(id));
        let signature = token;
        let json = match version {
            Version::V1 => {
                let public_key = From::from(public_key.as_slice());
                Builder::<V1, Public>::try_verify(signature, &public_key, footer)?
            },
            Version::V2 => {
                let key = rusty_paseto::core::Key::from(&public_key);
                let public_key = From::from(&key);
                Builder::<V2, Public>::try_verify(signature, &public_key, footer)?
            },
            Version::V4 => {
                let implicit_assertion = None;
                let key = rusty_paseto::core::Key::from(&public_key);
                let public_key = From::from(&key);
                Builder::<V4, Public>::try_verify(signature, &public_key, footer, implicit_assertion)?
            },
        };
        serde_json::from_str(&json).map_err(|_|{Error::InvalidToken})
    }
}


impl<Payload: Serialize + DeserializeOwned + Send + Sync + 'static> Tokenizer<Payload> for Paseto {
    fn try_sign<Input: Into<Payload>>(&self, input: Input) -> Result<String> {
        let payload = input.into();
        let payload = serde_json::to_string(&payload).map_err(Error::internal)?;
        let payload = payload.as_str();
        self.try_sign(payload)
    }

    fn try_verify(&self, token: &str) -> Result<Payload> {
        let (id, public_key, version) = (self.current.id.to_string(), self.current.public_key, self.current.version);
        let id = id.as_str();
        let err = match Self::try_verify(id, public_key, version, token) {
            Ok(payload) => return Ok(payload),
            Err(err) => err,
        };
        if let Some(key) = self.previous {
            let (id, public_key, version) = (key.id.to_string(), key.public_key, key.version);
            let id = id.as_str();
            return Self::try_verify(id, public_key, version, token);
        }
        Err(err)
    }
}


#[cfg(test)]
mod tests {
    use crate::domain::types::{User, Token, Contact, EmailAddress, Audience};
    use super::*;

    fn user() -> User {
        let id = Default::default();
        let username = String::from("user");
        let name = String::new();
        let contact = Contact::Email(EmailAddress::new("user@example.com", false).unwrap());
        let password = String::new();
        let login = Default::default();
        let profile = Default::default();

        User{id, username, name, contact, password, login, profile}
    }

    #[test]
    fn test_paseto_v4_sign_and_verify() {
        let user = user();
        let paseto = Paseto::default();
        let input = (&user, Audience::default(), String::new());
        let input: Token = input.into();
        let token = <Paseto as Tokenizer<Token>>::try_sign(&paseto, input.clone()).unwrap();
        let payload = <Paseto as Tokenizer<Token>>::try_verify(&paseto, token.as_str()).expect("Could Not verify the Paseto token");
        assert_eq!(input, payload);
    }

    #[test]
    fn test_serialized_paseto() {
        let paseto = Paseto::default();
        let json = serde_json::to_string(&paseto).unwrap();
        let paseto = serde_json::from_str(&json).unwrap();
        let user = user();
        let input = (&user, Audience::default(), String::new());
        let token = <Paseto as Tokenizer<Token>>::try_sign(&paseto, input).unwrap();
        let payload = <Paseto as Tokenizer<Token>>::try_verify(&paseto, token.as_str()).expect("Could Not verify the Paseto token");
    }
}
