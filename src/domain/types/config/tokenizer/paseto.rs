use rusty_paseto::core::{Paseto as Builder, Public, V1, V2, V4, Footer};
use serde::{Serialize, Deserialize, de::DeserializeOwned};
use crate::domain::services::{Tokenizer, Result};
use rand::{rngs::OsRng, CryptoRng, TryRngCore};
use ed25519_dalek::{SigningKey, SecretKey};
use crate::domain::types::Error;
use sha2::{Sha256, Digest};



#[derive(Copy, Clone, Debug, Serialize, Deserialize, Default)]
pub struct Paseto {
    current: Key,
    previous: Option<Key>
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct Key {
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


impl Key {
    fn id(key: &[u8]) -> u64 {
        let mut hasher = Sha256::new();
        hasher.update(key);
        let result = hasher.finalize();
        // Take the first 8 bytes of the hash for the u64 ID
        let bytes: [u8; 8] = result[0..8]
            .try_into()
            .expect("Failed to generate id from hash");
        u64::from_be_bytes(bytes)
    }

    fn keys() -> ([u8; 32], [u8; 32]) {
        // Generate a new signing key using a secure random number generator.
        let key = Self::generate();
        
        // Extract the public key from the signing key.
        let public_key = key.verifying_key().to_bytes();
        
        // Convert the signing key to a byte array for the private key.
        let private_key = key.to_bytes();
        (private_key, public_key)
    }

    fn generate() -> SigningKey {
        let mut csprng = OsRng;
        let mut secret = [0u8; 32];
        csprng.try_fill_bytes(&mut secret).expect("Failed to generate random bytes");
        SigningKey::from_bytes(&secret)
    }
}


impl Default for Key {
    fn default() -> Self {
        let version = Version::default();
        let (private_key, public_key) = Self::keys();
        let id = Self::id(public_key.as_slice());
        Self{id, version, private_key, public_key}
    }
}


impl Paseto {
    fn try_verify<Payload: DeserializeOwned + Send + Sync + 'static>(id: u64, key: [u8; 32], token: &str, version: Version) -> Result<Payload> {
        let id = id.to_string();
        let footer = Some(Footer::from(id.as_str()));
        match version {
            Version::V1 => {
                let public_key = From::from(key.as_slice());
                let json = Builder::<V1, Public>::try_verify(token, &public_key, footer)?;
                Ok(serde_json::from_str(&json).map_err(|_|Error::InvalidToken)?)
            },
            Version::V2 => {
                let key = rusty_paseto::core::Key::from(key);
                let public_key = From::from(&key);
                let json = Builder::<V2, Public>::try_verify(token, &public_key, footer)?;
                Ok(serde_json::from_str(&json).map_err(|_|Error::InvalidToken)?)
            },
            Version::V4 => {
                let key = rusty_paseto::core::Key::from(key);
                let public_key = From::from(&key);
                let implicit_assertion = None;
                let json = Builder::<V4, Public>::try_verify(token, &public_key, footer, implicit_assertion)?;
                Ok(serde_json::from_str(&json).map_err(|_|Error::InvalidToken)?)
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
                Ok(Builder::<V2, Public>::builder().set_payload(payload).set_footer(footer).try_sign(&key)?)
            },
            Version::V2 => {
                let key = From::from(key.as_slice());
                Ok(Builder::<V2, Public>::builder().set_payload(payload).set_footer(footer).try_sign(&key)?)
            },
            Version::V4 => {
                let key = From::from(key.as_slice());
                Ok(Builder::<V2, Public>::builder().set_payload(payload).set_footer(footer).try_sign(&key)?)
            },
        }
    }

    fn try_verify(&self, token_str: &str) -> Result<Payload> {
        todo!()
    }
}