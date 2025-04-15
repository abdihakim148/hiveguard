use rusty_paseto::core::{PasetoAsymmetricPrivateKey, PasetoAsymmetricPublicKey, PasetoSymmetricKey, V4, V3, V2, V1, Public, Local, VersionTrait, Payload, Footer, Paseto as Builder};
use serde::de::{self, Deserializer, DeserializeOwned};
use rand::{rngs::OsRng, CryptoRng, TryRngCore};
use ed25519_dalek::{SecretKey, SigningKey};
use crate::domain::services::Tokenizer;
use serde::{Deserialize, Serialize};
use crate::domain::types::Error;
use chrono::{DateTime, Utc};
use sha2::{Digest, Sha256};

#[derive(Copy, Clone, Debug, Default, Serialize, Deserialize)]
pub struct Paseto {
    current: Key,
    previous: Option<Key>,
}

#[derive(Copy, Clone, Debug, Default, Serialize, Deserialize)]
pub struct Key {
    version: Version,
    #[serde(flatten)]
    purpose: Purpose, // Purpose will use manual Deserialize now
}

#[derive(Copy, Clone, Debug, Default, Serialize, Deserialize)]
pub enum Version {
    #[default]
    V4,
    // V3,
    V2,
    V1,
}

#[derive(Copy, Clone, Debug, Serialize)] // Keep Serialize
#[serde(tag = "purpose", content = "content")]
pub enum Purpose {
    #[serde(rename = "public")]
    Public {
        #[serde(skip)]
        id: u64,
        private_key: [u8; 32],
        public_key: [u8; 32],
        created_at: DateTime<Utc>,
    },
    #[serde(rename = "local")]
    Local {
        #[serde(skip)]
        id: u64,
        key: [u8; 32],
        created_at: DateTime<Utc>,
    },
}

impl Default for Purpose {
    fn default() -> Self {
        let mut csprng = OsRng;
        let mut secret = [0u8; 32];
        csprng
            .try_fill_bytes(&mut secret)
            .expect("Failed to generate random bytes");
        let key = SigningKey::from_bytes(&secret);
        let public_key = key.verifying_key().to_bytes();
        let private_key = key.to_bytes();
        let created_at = Utc::now();
        // Default still generates ID from public key
        let id = Self::generate_id(&public_key);
        Self::Public {
            id,
            private_key,
            public_key,
            created_at,
        }
    }
}

impl Purpose {
    // This function is to generate a `u64` from the key's hash value
    fn generate_id(key: &[u8]) -> u64 {
        let mut hasher = Sha256::new();
        hasher.update(key);
        let result = hasher.finalize();
        // Take the first 8 bytes of the hash for the u64 ID
        let bytes: [u8; 8] = result[0..8]
            .try_into()
            .expect("Failed to generate id from hash");
        u64::from_be_bytes(bytes)
    }

    pub fn id(&self) -> &u64 {
        match self {
            Self::Public { id, private_key, public_key, created_at } => id,
            Self::Local { id, key, created_at } => id,
        }
    }
}

// Define InnerPurpose outside the function for clarity and potentially fixing inference issues
// This helper enum mirrors the JSON structure for deserialization purposes.
#[derive(Deserialize)] // Only need Deserialize here
#[serde(tag = "purpose", content = "content")]
enum InnerPurpose {
    // We capture the fields from the JSON content block.
    // The `id` is not expected in the JSON content for `Purpose`,
    // as it's derived, so we don't include it here.
    #[serde(alias = "public")]
    Public {
        private_key: [u8; 32],
        public_key: [u8; 32],
        created_at: DateTime<Utc>,
    },
    #[serde(alias = "local")]
    Local {
        key: [u8; 32],
        created_at: DateTime<Utc>,
    },
}

// Manual Deserialize implementation for Purpose
impl<'de> Deserialize<'de> for Purpose {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        // Deserialize into the helper enum first
        let inner_purpose = InnerPurpose::deserialize(deserializer)?;

        // Match on the helper enum and construct the actual Purpose enum, generating the ID
        match inner_purpose {
            InnerPurpose::Public {
                private_key,
                public_key,
                created_at,
            } => {
                // Generate the ID based on the public key for the Public variant
                let id = Self::generate_id(&public_key);
                Ok(Purpose::Public {
                    id,
                    private_key,
                    public_key,
                    created_at,
                })
            }
            InnerPurpose::Local { key, created_at } => {
                // Generate the ID based on the symmetric key for the Local variant
                let id = Self::generate_id(&key);
                Ok(Purpose::Local {
                    id,
                    key,
                    created_at,
                })
            }
        }
        // The match expression is the last expression, so its Result<Purpose, D::Error> is returned implicitly
    }
}


impl Paseto {
    fn sign<'a>(&'a self, id: u64, key: &[u8], payload: impl Into<Payload<'a>>) -> Result<String, Error> {
        let payload = payload.into();
        let id = id.to_string();
        let footer = Footer::from(id.as_str());
        match &self.current.version {
            Version::V1 => {
                let key = From::from(key);
                Ok(Builder::<V1, Public>::builder().set_payload(payload).set_footer(footer).try_sign(&key)?)
            },
            Version::V2 => {
                let key = From::from(key);
                Ok(Builder::<V2, Public>::builder().set_payload(payload).set_footer(footer).try_sign(&key)?)
            },
            Version::V4 => {
                let key = From::from(key);
                Ok(Builder::<V4, Public>::builder().set_payload(payload).set_footer(footer).try_sign(&key)?)
            },
        }
    }


    fn verify<Payload: DeserializeOwned + Send + Sync + 'static>(&self, token: &str, current_key: (u64, [u8; 32]), previous_key: Option<(u64, [u8; 32])>) -> Result<Payload, Error> {
        let (id, key) = (current_key.0.to_string(), current_key.1);
        let footer = Some(Footer::from(id.as_str()));
        let implicit_assertion = None;
        match &self.current.version {
            Version::V1 => {
                let public_key = From::from(key.as_slice());
                match Builder::<V1, Public>::try_verify(token, &public_key, footer) {
                    Ok(s) => return serde_json::from_str(&s).map_err(|_|{Error::InvalidToken}),
                    Err(err) => {
                        use rusty_paseto::core::PasetoError;
                        match &err {
                            PasetoError::FooterInvalid => (),
                            _ => return Err(err)?
                        }
                    }
                }
            },
            Version::V2 => {
                let key = rusty_paseto::core::Key::from(key);
                let public_key = From::from(&key);
                match Builder::<V2, Public>::try_verify(token, &public_key, footer) {
                    Ok(s) => return serde_json::from_str(&s).map_err(|_|{Error::InvalidToken}),
                    Err(err) => {
                        use rusty_paseto::core::PasetoError;
                        match &err {
                            PasetoError::FooterInvalid => (),
                            _ => return Err(err)?
                        }
                    }
                }
            },
            Version::V4 => {
                let key = rusty_paseto::core::Key::from(key);
                let public_key = From::from(&key);
                match Builder::<V4, Public>::try_verify(token, &public_key, footer, implicit_assertion) {
                    Ok(s) => return serde_json::from_str(&s).map_err(|_|{Error::InvalidToken}),
                    Err(err) => {
                        use rusty_paseto::core::PasetoError;
                        match &err {
                            PasetoError::FooterInvalid => (),
                            _ => return Err(err)?
                        }
                    }
                }
            },
        }
        if let Some(previous_key) = previous_key {
            let (id, key) = (previous_key.0.to_string(), previous_key.1);
            let footer = Some(Footer::from(id.as_str()));
            let version = match &self.previous{Some(key) => key.version, None => return Err(Error::internal("This is never supposed to happend. wrong code branching during paseto public token verification."))};
            match version {
                Version::V1 => {
                    let public_key = From::from(key.as_slice());
                    let json = Builder::<V1, Public>::try_verify(token, &public_key, footer)?;
                    return serde_json::from_str(&json).map_err(|_| Error::InvalidToken)
                },
                Version::V2 => {
                    let key = rusty_paseto::core::Key::from(key);
                    let public_key = From::from(&key);
                    let json = Builder::<V2, Public>::try_verify(token, &public_key, footer)?;
                    return serde_json::from_str(&json).map_err(|_| Error::InvalidToken)
                },
                Version::V4 => {
                    let key = rusty_paseto::core::Key::from(key);
                    let public_key = From::from(&key);
                    let json = Builder::<V4, Public>::try_verify(token, &public_key, footer, implicit_assertion)?;
                    return serde_json::from_str(&json).map_err(|_| Error::InvalidToken)
                },
            }
        }
        Err(Error::InvalidToken)
    }
}



impl<Payload: Serialize + DeserializeOwned + Send + Sync + 'static> Tokenizer<Payload> for Paseto {
    fn try_sign<Input: Into<Payload>>(&self, input: Input) -> crate::domain::services::Result<String> {
        todo!()
    }

    fn try_verify(&self, token_str: &str) -> Result<Payload, Error> {
        todo!()
    }
}




#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{TimeZone, Utc};
    use serde_json;

    // Helper to create a fixed DateTime<Utc> for consistent testing
    fn fixed_utc_datetime() -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2025, 4, 15, 20, 28, 10).unwrap() // Corresponds to 11:28:10 PM UTC+3
    }

    // Helper to create a sample Paseto struct for testing
    fn create_sample_paseto() -> Paseto {
        let created_at = fixed_utc_datetime();
        // Use fixed keys for predictable ID generation
        let public_key = [1u8; 32];
        let private_key = [2u8; 32];
        let id = Purpose::generate_id(&public_key); // Generate ID based on the fixed public key

        Paseto {
            current: Key {
                version: Version::V4,
                purpose: Purpose::Public {
                    id, // Use the generated ID
                    private_key,
                    public_key,
                    created_at,
                },
            },
            previous: None,
        }
    }

    #[test]
    fn test_paseto_serialization() {
        let paseto_instance = create_sample_paseto();
        let serialized =
            serde_json::to_value(&paseto_instance).expect("Failed to serialize Paseto");

        // Expected JSON structure (values will differ for keys/id)
        let expected_structure = serde_json::json!({
            "current": {
                "version": "V4", // Matches Version::V4 serialization
                "purpose": "public", // Matches the tag for Purpose::Public
                "content": { // Matches the content field for Purpose::Public
                    // id is NOT serialized here due to #[serde(skip)] in InnerPurpose
                    "private_key": paseto_instance.current.purpose.private_key(), // Use actual values
                    "public_key": paseto_instance.current.purpose.public_key(),
                    "created_at": paseto_instance.current.purpose.created_at().to_rfc3339(), // Use actual values formatted
                }
            },
            "previous": null
        });

        // Compare the structure and specific fields
        assert_eq!(
            serialized["current"]["version"],
            expected_structure["current"]["version"]
        );
        assert_eq!(
            serialized["current"]["purpose"],
            expected_structure["current"]["purpose"]
        );
        assert_eq!(
            serialized["current"]["content"]["private_key"],
            expected_structure["current"]["content"]["private_key"]
        );
        assert_eq!(
            serialized["current"]["content"]["public_key"],
            expected_structure["current"]["content"]["public_key"]
        );
        // Compare timestamps by parsing them back to DateTime<Utc>
        let serialized_dt_str = serialized["current"]["content"]["created_at"].as_str().unwrap();
        let expected_dt_str = expected_structure["current"]["content"]["created_at"].as_str().unwrap();

        let serialized_dt = DateTime::parse_from_rfc3339(serialized_dt_str).unwrap().with_timezone(&Utc);
        let expected_dt = DateTime::parse_from_rfc3339(expected_dt_str).unwrap().with_timezone(&Utc);

        assert_eq!(serialized_dt, expected_dt, "Timestamps do not match");
        assert_eq!(serialized["previous"], expected_structure["previous"]);
    }

    #[test]
    fn test_paseto_deserialization_public() {
        let created_at_str = fixed_utc_datetime().to_rfc3339();
        // Use fixed keys matching the helper function
        let public_key_arr = [1u8; 32];
        let private_key_arr = [2u8; 32];
        let expected_id = Purpose::generate_id(&public_key_arr); // Calculate expected ID

        // Serialize byte arrays correctly to JSON arrays
        let private_key_json = serde_json::to_string(&private_key_arr.to_vec()).unwrap();
        let public_key_json = serde_json::to_string(&public_key_arr.to_vec()).unwrap();

        let json_data = format!(
            r#"{{
                    "current": {{
                        "version": "V4",
                        "purpose": "public",
                        "content": {{
                            "private_key": {},
                            "public_key": {},
                            "created_at": "{}"
                        }}
                    }},
                    "previous": null
                }}"#,
            private_key_json, // Use the JSON string
            public_key_json,  // Use the JSON string
            created_at_str
        );

        let deserialized: Paseto =
            serde_json::from_str(&json_data).expect("Failed to deserialize Paseto");

        // Assertions
        assert!(matches!(deserialized.current.version, Version::V4));
        assert!(deserialized.previous.is_none());

        match deserialized.current.purpose {
            Purpose::Public {
                id,
                private_key,
                public_key,
                created_at,
            } => {
                assert_eq!(
                    id, expected_id,
                    "Deserialized ID does not match expected generated ID"
                );
                assert_eq!(private_key, private_key_arr);
                assert_eq!(public_key, public_key_arr);
                assert_eq!(created_at, fixed_utc_datetime());
            }
            _ => panic!("Deserialized into wrong Purpose variant"),
        }
    }

    #[test]
    fn test_paseto_deserialization_local() {
        let created_at_str = fixed_utc_datetime().to_rfc3339();
        let key_arr = [3u8; 32]; // Different key for local
        let expected_id = Purpose::generate_id(&key_arr); // Calculate expected ID for local key

        // Serialize byte array correctly to a JSON array
        let key_json = serde_json::to_string(&key_arr).unwrap();

        let json_data = format!(
            r#"{{
                    "current": {{
                        "version": "V2",
                        "purpose": "local",
                        "content": {{
                            "key": {},
                            "created_at": "{}"
                        }}
                    }},
                    "previous": null
                }}"#,
            key_json, // Use the JSON string
            created_at_str
        );

        let deserialized: Paseto = serde_json::from_str(&json_data)
            .unwrap_or_else(|e| panic!("Failed to deserialize Paseto: {}", e));

        // Assertions
        assert!(matches!(deserialized.current.version, Version::V2));
        assert!(deserialized.previous.is_none());

        match deserialized.current.purpose {
            Purpose::Local {
                id,
                key,
                created_at,
            } => {
                assert_eq!(
                    id, expected_id,
                    "Deserialized ID does not match expected generated ID for local key"
                );
                assert_eq!(key, key_arr);
                assert_eq!(created_at, fixed_utc_datetime());
            }
            _ => panic!("Deserialized into wrong Purpose variant"),
        }
    }

    // Add a helper method to Purpose for tests to access fields easily
    // This avoids making fields pub(crate) or similar just for tests.
    impl Purpose {
        fn private_key(&self) -> [u8; 32] {
            match self {
                Purpose::Public { private_key, .. } => *private_key,
                _ => panic!("Called private_key on non-Public Purpose"),
            }
        }

        fn public_key(&self) -> [u8; 32] {
            match self {
                Purpose::Public { public_key, .. } => *public_key,
                _ => panic!("Called public_key on non-Public Purpose"),
            }
        }

        fn key(&self) -> [u8; 32] {
            match self {
                Purpose::Local { key, .. } => *key,
                _ => panic!("Called key on non-Local Purpose"),
            }
        }

        fn created_at(&self) -> DateTime<Utc> {
            match self {
                Purpose::Public { created_at, .. } => *created_at,
                Purpose::Local { created_at, .. } => *created_at,
            }
        }
    }
}
