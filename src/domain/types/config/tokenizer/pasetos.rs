use crate::domain::services::{Result, Tokenizer};
use crate::domain::types::Error;
use crate::domain::types::PasetoKeys;
use ed25519_dalek::{SecretKey, SigningKey};
use argon2::password_hash::rand_core::{OsRng, CryptoRngCore};
use rusty_paseto::core::{Footer, Paseto as Builder, Public, V1, V2, V4};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Paseto {
    issuer: String,
    audience: String,
    token_ttl_seconds: i64,
    refresh_token_ttl_seconds: i64,
    current: Key,
    previous: Option<Key>,
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct Key {
    //#[serde(skip)]
    id: u64,
    version: Version,
    private_key: [u8; 32],
    public_key: [u8; 32],
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize, Default)]
pub enum Version {
    #[default]
    V4,
    V2,
    V1,
}

impl Default for Paseto {
    fn default() -> Self {
        let issuer = String::from(crate::NAME);
        let audience = String::from(crate::NAME);
        let token_ttl_seconds = 15 * 60;
        let refresh_token_ttl_seconds = 30 * 24 * 60 * 60;
        let current = Key::default();
        let previous = None;
        Self {
            issuer,
            audience,
            token_ttl_seconds,
            refresh_token_ttl_seconds,
            current,
            previous,
        }
    }
}

impl Key {
    fn set_id(&mut self) {
        let mut hasher = Sha256::new();
        hasher.update(&self.public_key);
        let result = hasher.finalize();
        // Take the first 8 bytes of the hash for the u64 ID
        let bytes: [u8; 8] = result[0..8]
            .try_into()
            .expect("Failed to generate id from hash");
        self.id = u64::from_be_bytes(bytes);
    }

    fn set_keys(&mut self) {
        // Generate a new signing key using a secure random number generator.
        let signing_key = Self::generate();

        //Extract the private key from the signing key.
        self.private_key = signing_key.to_bytes();

        // Extract the public key from the signing key.
        self.public_key = signing_key.verifying_key().to_bytes();
    }

    fn generate() -> SigningKey {
        let mut csprng = OsRng;
        //let mut secret = [0u8; 32];
        //csprng.try_fill_bytes(&mut secret).expect("Failed to generate random bytes");
        //SigningKey::from_bytes(&secret);
        SigningKey::generate(&mut csprng)
    }
}

impl Default for Key {
    fn default() -> Self {
        let id = 0;
        let private_key = Default::default();
        let public_key = Default::default();
        let version = Default::default();
        let mut key = Self{id, private_key, public_key, version};
        key.set_keys();
        key.set_id();
        key
    }
}

impl From<PasetoKeys> for Key {
    fn from(value: PasetoKeys) -> Self {
        let version = Default::default();
        let private_key = value.private_key;
        let public_key = value.public_key;
        let id = 0;
        let mut key = Key {
            id,
            private_key,
            public_key,
            version,
        };
        key.set_id();
        key
    }
}

impl Paseto {
    fn try_verify<Payload: DeserializeOwned + Send + Sync + 'static>(
        id: u64,
        key: [u8; 32],
        token: &str,
        version: Version,
    ) -> Result<Payload> {
        let id = id.to_string();
        let footer = None;//Some(Footer::from(id.as_str()));
        match version {
            Version::V1 => {
                let public_key = From::from(key.as_slice());
                let json = Builder::<V1, Public>::try_verify(token, &public_key, footer)?;
                Ok(serde_json::from_str(&json).map_err(|_| Error::InvalidToken)?)
            }
            Version::V2 => {
                let key = rusty_paseto::core::Key::from(key);
                let public_key = From::from(&key);
                let json = Builder::<V2, Public>::try_verify(token, &public_key, footer)?;
                Ok(serde_json::from_str(&json).map_err(|_| Error::InvalidToken)?)
            }
            Version::V4 => {
                let key = rusty_paseto::core::Key::from(&key);
                let public_key = From::from(&key);
                let implicit_assertion = None;
                let json = Builder::<V4, Public>::try_verify(
                    token,
                    &public_key,
                    footer,
                    implicit_assertion,
                )?;
                Ok(serde_json::from_str(&json).map_err(|_| Error::InvalidToken)?)
            }
        }
    }
}

impl<Payload: Serialize + DeserializeOwned + Send + Sync + 'static> Tokenizer<Payload> for Paseto {
    fn try_sign<Input: Into<Payload>>(&self, input: Input) -> Result<String> {
        let payload = input.into();
        let json = serde_json::to_string(&payload)?;
        let payload = rusty_paseto::core::Payload::from(json.as_str());
        let mut key = [0u8; 64];
        let id = self.current.id.to_string();
        key[..32].copy_from_slice(&self.current.private_key);
        key[32..].copy_from_slice(&self.current.public_key);
        let footer = Footer::from(id.as_str());
        match self.current.version {
            Version::V1 => {
                let key = From::from(key.as_slice());
                Ok(Builder::<V2, Public>::builder()
                    .set_payload(payload)
                    .set_footer(footer)
                    .try_sign(&key)?)
            }
            Version::V2 => {
                let key = From::from(key.as_slice());
                Ok(Builder::<V2, Public>::builder()
                    .set_payload(payload)
                    .set_footer(footer)
                    .try_sign(&key)?)
            }
            Version::V4 => {
                let key = rusty_paseto::core::Key::from(&key);
                let private_key = From::from(&key);
                Ok(Builder::<V4, Public>::builder()
                    .set_payload(payload)
                    //.set_footer(footer)
                    .set_implicit_assertion(rusty_paseto::core::ImplicitAssertion::from("")) // Provide empty assertion
                    .try_sign(&private_key)?)
            }
        }
    }

    fn try_verify(&self, token: &str) -> Result<Payload> {
        let (id, key, version) = (
            self.current.id,
            self.current.public_key,
            self.current.version,
        );
        let err = match Paseto::try_verify(id, key, token, version) {
            Ok(payload) => return Ok(payload),
            Err(err) => err,
        };
        if let Some(key) = self.previous {
            let (id, key, version) = (key.id, key.public_key, key.version);
            return Paseto::try_verify(id, key, token, version);
        }
        Err(err)?
    }
}
