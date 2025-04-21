use rand::{rngs::OsRng, CryptoRng, TryRngCore};
use serde::{Serialize, Deserialize};
use ed25519_dalek::SigningKey;
use sha2::{Sha256, Digest};


#[derive(Copy, Clone, Debug, Serialize, Deserialize, Default, PartialEq)]
pub enum Version {
    #[default]
    V4,
    V2,
    V1,
}


#[derive(Copy, Clone, Debug, Serialize, PartialEq)]
pub struct Key {
    #[serde(skip)]
    pub id: u64,
    pub version: Version,
    pub private_key: [u8; 32],
    pub public_key: [u8; 32],
}


#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct Paseto {
    pub issuer: String,
    pub audience: String,
    pub token_ttl_seconds: i64,
    pub refresh_token_ttl_seconds: i64,
    pub current: Key,
    pub previous: Option<Key>,
}

impl Key {
    pub fn id(public_key: &[u8; 32]) -> u64 {
        let mut hasher = Sha256::new();
        hasher.update(public_key);
        let result = hasher.finalize();
        // Take the first 8 bytes of the hash for the u64 ID
        let bytes: [u8; 8] = result[0..8]
            .try_into()
            .expect("Failed to generate id from hash");
        u64::from_be_bytes(bytes)
    }

    fn generate() -> ([u8; 32], [u8; 32]) {
        let mut csprng = OsRng;
        let mut secret = [0u8; 32];
        csprng.try_fill_bytes(&mut secret).expect("Failed to generate random bytes");
        let signing_key = SigningKey::from_bytes(&secret);
        let private_key = signing_key.to_bytes();
        let public_key = signing_key.verifying_key().to_bytes();
        (private_key, public_key)
    }
}



impl Default for Key {
    fn default() -> Self {
        let (private_key, public_key) = Self::generate();
        let id = Self::id(&public_key);
        let version = Default::default();
        Self{id, private_key, public_key, version}
    }
}
