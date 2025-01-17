use chrono::Utc;
use rusty_paseto::core::{ImplicitAssertion, PasetoError, Payload, Public, V4};
use rusty_paseto::core::PasetoAsymmetricPrivateKey;
use rusty_paseto::core::PasetoAsymmetricPublicKey;
use rusty_paseto::core::Paseto as PasetoBuilder;
use serde::{Serialize, de::DeserializeOwned};
use super::super::types::{Error, Token, PasetoKeys, Value};
use rusty_paseto::core::Footer;
use rusty_paseto::core::Key;


type Result<T> = std::result::Result<T, Error>;
pub trait Paseto: Serialize + DeserializeOwned + 'static {
    fn expired(&self) -> bool;

    fn try_verify(signature: &str, keys: &PasetoKeys) -> Result<Self> {
        let key = Key::from(&keys.public_key);
        let public_key = From::from(&key);
        let footer = Option::<Footer>::None;
        let implicit_assertion = Option::<ImplicitAssertion>::None;
        let json = match PasetoBuilder::try_verify(signature, &public_key, footer, implicit_assertion) {
            Ok(value) => value,
            Err(err) => {
                match err {
                    PasetoError::InvalidSignature => {
                        let key = match keys.prev_public_key {
                            Some(key) => Key::from(key),
                            None => Err(err)?
                        };
                        let public_key = From::from(&key);
                        PasetoBuilder::try_verify(signature, &public_key, footer, implicit_assertion)?
                    },
                    _ => Err(err)?
                }
            }
        };
        match serde_json::from_str(&json) {
            Ok(value) => Ok(value),
            Err(err) => Err(Error::InvalidToken)
        }
    }


    fn try_sign(&self, keys: &PasetoKeys) -> Result<String> {
        let mut key = [0u8; 64];
        key[..32].copy_from_slice(&keys.private_key);
        key[32..].copy_from_slice(&keys.public_key);
        let key = Key::from(&key);
        let key = From::from(&key);
        let json = serde_json::to_string(&self).map_err(|err|Error::<Value>::InternalServerError(err.into()))?;
        let payload = Payload::from(json.as_str());
        let token = PasetoBuilder::<V4, Public>::builder().set_payload(payload).try_sign(&key)?;
        Ok(token)
    }
}


impl Paseto for Token {
    fn expired(&self) -> bool {
        if let Some(expiration) = &self.expiration {
            return Utc::now() >= *expiration
        }
        false
    }
}