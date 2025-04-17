use std::collections::HashSet;
use serde::{Serialize, Deserialize, de::{DeserializeOwned, Deserializer, Visitor, MapAccess, Error as SerdeError}};
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation, encode, decode, decode_header, TokenData};
use crate::domain::services::{Tokenizer, Result};
use crate::domain::types::Error;
use chrono::{Utc, Duration};
use sha2::{Sha256, Digest};
use base64::prelude::*;
use std::fmt;


#[derive(Debug, Serialize, Deserialize)]
struct Claims<T> {
    // aud: String, // Audience - Now part of standard validation
    exp: i64, // Required (validate_exp defaults to true in validation). Expiration time (as UTC timestamp)
    iat: i64, // Optional. Issued at (as UTC timestamp)
    // iss: String, // Issuer - Now part of standard validation
    // nbf: usize, // Optional. Not Before (as UTC timestamp)
    // sub: String, // Optional. Subject (whom token refers to)
    payload: T, // Custom payload
}


#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Jwt{
    issuer: String,
    audience: String,
    token_ttl_seconds: i64,
    refresh_token_ttl_seconds: i64,
    current: Key,
    previous: Option<Key>
}

#[derive(Clone, Debug, Serialize)]
pub struct Key {
    id: u64,
    algorithm: Algorithm,
    private_key: Vec<u8>,
    public_key: Vec<u8>
}

impl Key {
    // Helper function to generate ID from public key
    fn generate_id(public_key: &[u8]) -> u64 {
        let mut hasher = Sha256::new();
        hasher.update(public_key);
        let result = hasher.finalize();
        // Take the first 8 bytes of the hash for the u64 ID
        let bytes: [u8; 8] = result[0..8]
            .try_into()
            .expect("Failed to generate id from hash: slice length mismatch");
        u64::from_be_bytes(bytes)
    }
}

// Manual Deserialize implementation for Key
impl<'de> Deserialize<'de> for Key {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(field_identifier, rename_all = "snake_case")]
        enum Field { Algorithm, PrivateKey, PublicKey }

        struct KeyVisitor;

        impl<'de> Visitor<'de> for KeyVisitor {
            type Value = Key;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct Key")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<Key, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut algorithm = None;
                let mut private_key: Option<Vec<u8>> = None;
                let mut public_key: Option<Vec<u8>> = None;

                while let Some(key) = map.next_key()? {
                    match key {
                        Field::Algorithm => {
                            if algorithm.is_some() {
                                return Err(SerdeError::duplicate_field("algorithm"));
                            }
                            algorithm = Some(map.next_value()?);
                        }
                        Field::PrivateKey => {
                            if private_key.is_some() {
                                return Err(SerdeError::duplicate_field("private_key"));
                            }
                            // Deserialize private_key as base64 encoded string
                            let base64_str: String = map.next_value()?;
                            private_key = Some(BASE64_STANDARD.decode(&base64_str).map_err(|e| SerdeError::custom(format!("Failed to decode private_key from base64: {}", e)))?);
                        }
                        Field::PublicKey => {
                            if public_key.is_some() {
                                return Err(SerdeError::duplicate_field("public_key"));
                            }
                             // Deserialize public_key as base64 encoded string
                            let base64_str: String = map.next_value()?;
                            public_key = Some(BASE64_STANDARD.decode(&base64_str).map_err(|e| SerdeError::custom(format!("Failed to decode public_key from base64: {}", e)))?);
                        }
                    }
                }

                let algorithm = algorithm.ok_or_else(|| SerdeError::missing_field("algorithm"))?;
                let private_key = private_key.ok_or_else(|| SerdeError::missing_field("private_key"))?;
                let public_key = public_key.ok_or_else(|| SerdeError::missing_field("public_key"))?;

                // Generate the ID from the public key
                let id = Key::generate_id(&public_key);

                Ok(Key { id, algorithm, private_key, public_key })
            }
        }

        const FIELDS: &'static [&'static str] = &["algorithm", "private_key", "public_key"];
        deserializer.deserialize_struct("Key", FIELDS, KeyVisitor)
    }
}


impl TryFrom<&Key> for EncodingKey {
    type Error = Error;
    fn try_from(key: &Key) -> std::result::Result<Self, Self::Error> {
        match &key.algorithm {
            Algorithm::RS256 | Algorithm::RS384 | Algorithm::RS512 | 
            Algorithm::PS256 | Algorithm::PS384 | Algorithm::PS512 => {
                Ok(EncodingKey::from_rsa_pem(&key.private_key)?)
            },
            Algorithm::ES256 | Algorithm::ES384 => {
                Ok(EncodingKey::from_ec_pem(&key.private_key)?)
            },
            Algorithm::EdDSA => {
                Ok(EncodingKey::from_ed_pem(&key.private_key)?)
            },
            _ => {
                Err(Error::UnknownAlgorithm)
            }
        }
    }
}

// Add TryFrom for DecodingKey
impl TryFrom<&Key> for DecodingKey {
    type Error = Error;
    fn try_from(key: &Key) -> std::result::Result<Self, Self::Error> {
        match &key.algorithm {
            Algorithm::RS256 | Algorithm::RS384 | Algorithm::RS512 |
            Algorithm::PS256 | Algorithm::PS384 | Algorithm::PS512 => {
                Ok(DecodingKey::from_rsa_pem(&key.public_key)?)
            },
            Algorithm::ES256 | Algorithm::ES384 => {
                Ok(DecodingKey::from_ec_pem(&key.public_key)?)
            },
            Algorithm::EdDSA => {
                Ok(DecodingKey::from_ed_pem(&key.public_key)?)
            },
            _ => {
                Err(Error::UnknownAlgorithm)
            }
        }
    }
}


impl Jwt {
    fn try_sign<Payload: Serialize>(&self, payload: Payload) -> Result<String> {
        let now = Utc::now();
        let iat = now.timestamp();
        let exp = (now + Duration::seconds(self.token_ttl_seconds)).timestamp();

        let claims = Claims {
            // aud: self.audience.clone(), // Handled by validation
            // iss: self.issuer.clone(), // Handled by validation
            iat,
            exp,
            payload,
        };

        // Include kid in the header
        let mut header = Header::new(self.current.algorithm);
        header.kid = Some(self.current.id.to_string());

        let encoding_key = EncodingKey::try_from(&self.current)?;

        encode(&header, &claims, &encoding_key).map_err(Error::from)
    }

    fn try_verify<Payload: DeserializeOwned>(&self, token_str: &str) -> Result<Payload> {
        // 1. Decode header to get kid and algorithm
        let header = decode_header(token_str)?;
        let kid_str = header.kid.ok_or(Error::InvalidToken)?; // Ensure kid exists
        let kid = kid_str.parse::<u64>().map_err(|_| Error::InvalidToken)?; // Parse kid to u64
        let alg = header.alg;

        // 2. Find the key corresponding to the kid
        let key_to_use = if self.current.id == kid && self.current.algorithm == alg {
            &self.current
        } else if let Some(prev_key) = &self.previous {
            if prev_key.id == kid && prev_key.algorithm == alg {
                prev_key
            } else {
                return Err(Error::InvalidToken); // No matching key found
            }
        } else {
            return Err(Error::InvalidToken); // No previous key and current doesn't match
        };

        // 3. Set up validation
        let mut validation = Validation::new(key_to_use.algorithm); // Use algorithm from the identified key
        validation.set_issuer(&[self.issuer.clone()]);
        validation.set_audience(&[self.audience.clone()]);
        // Keep other defaults like validate_exp = true

        // 4. Get the decoding key
        let decoding_key = DecodingKey::try_from(key_to_use)?;

        // 5. Decode and validate the token
        decode::<Claims<Payload>>(token_str, &decoding_key, &validation)
            .map(|token_data| token_data.claims.payload)
            .map_err(Error::from)
    }
}


impl<Payload: Serialize + DeserializeOwned + Send + Sync + 'static> Tokenizer<Payload> for Jwt {
    fn try_sign<Input: Into<Payload>>(&self, input: Input) -> Result<String> {
        self.try_sign(input.into())
    }

    fn try_verify(&self, token_str: &str) -> Result<Payload> {
        self.try_verify(token_str)
    }
}