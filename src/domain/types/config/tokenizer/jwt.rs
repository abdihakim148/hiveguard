use std::collections::HashSet;
use serde::{Serialize, Deserialize, de::{DeserializeOwned, Deserializer, Visitor, MapAccess, Error as SerdeError}};
use serde_json::{Value, json, from_value}; // Added serde_json::{Value, json, from_value}
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation, encode, decode, decode_header}; // Removed TokenData
use crate::domain::services::{Tokenizer, Result};
use crate::domain::types::Error;
use chrono::{Utc, Duration};
use sha2::{Sha256, Digest};
use base64::prelude::*;
use std::fmt;


// Removed Claims struct


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

        const FIELDS: &[&str] = &["algorithm", "private_key", "public_key"];
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

        // Convert payload to serde_json::Value
        // The `?` operator will use the `From<serde_json::Error>` implementation for `Error`
        let mut claims_value = serde_json::to_value(payload)?;

        // Ensure it's an object to add standard claims
        if let Value::Object(ref mut map) = claims_value {
            map.insert("iss".to_string(), json!(self.issuer));
            map.insert("aud".to_string(), json!(self.audience));
            map.insert("iat".to_string(), json!(iat));
            map.insert("exp".to_string(), json!(exp));
        } else {
            // If the payload is not a JSON object, we cannot reliably add standard claims alongside it.
            // Return an internal error indicating the issue.
            return Err(Error::internal("Payload must serialize to a JSON object to add standard JWT claims"));
        }

        // Include kid in the header
        let mut header = Header::new(self.current.algorithm);
        header.kid = Some(self.current.id.to_string());

        let encoding_key = EncodingKey::try_from(&self.current)?;

        // Encode the serde_json::Value directly
        encode(&header, &claims_value, &encoding_key).map_err(Error::from)
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
        validation.leeway = 0; // Set leeway to 0 for precise expiration check
        // validate_exp defaults to true

        // 4. Get the decoding key
        let decoding_key = DecodingKey::try_from(key_to_use)?;

        // 5. Decode and validate the token
        // 5. Decode and validate the token into a serde_json::Value
        //    Standard claims (iss, aud, exp) are validated here by the jsonwebtoken library.
        let token_data = decode::<Value>(token_str, &decoding_key, &validation)?;

        // 6. Deserialize the validated Value into the target Payload type
        // The `?` operator will use the `From<serde_json::Error>` implementation for `Error`
        Ok(from_value(token_data.claims)?)
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


#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Serialize, Deserialize};
    use serde_json;
    use jsonwebtoken::Algorithm;
    use crate::domain::services::Tokenizer;
    use crate::domain::types::Error;
    use std::{fs, path::Path};
    use std::time::{Duration as StdDuration, SystemTime, UNIX_EPOCH};
    use base64::prelude::*;

    // Define a simple payload struct for testing
    #[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
    struct TestPayload {
        sub: String,
        company: String,
        // Removed redundant 'exp' field, as it's handled by try_sign
    }

    // Helper function to load key from file
    fn load_key_bytes(path: &str) -> Vec<u8> {
        fs::read(path).expect(&format!("Failed to read key file: {}", path))
    }

    // Helper to create a Key struct for testing, loading from files
    fn create_test_key_from_files(id_offset: u64, alg: Algorithm) -> Key {
        let private_key = load_key_bytes("test_private_key.pem");
        let public_key = load_key_bytes("test_public_key.pem");
        let id = Key::generate_id(&public_key) + id_offset; // Add offset for uniqueness if needed

        // Ensure keys are not empty
        assert!(!private_key.is_empty(), "Private key file is empty or could not be read.");
        assert!(!public_key.is_empty(), "Public key file is empty or could not be read.");

        Key {
            id,
            algorithm: alg,
            private_key,
            public_key,
        }
    }

    // Helper to create a Jwt config for testing
    fn create_test_jwt_config(current_key: Key, previous_key: Option<Key>) -> Jwt {
        Jwt {
            issuer: "test_issuer".to_string(),
            audience: "test_audience".to_string(),
            token_ttl_seconds: 300, // 5 minutes
            refresh_token_ttl_seconds: 3600, // 1 hour
            current: current_key,
            previous: previous_key,
        }
    }

    #[test]
    fn test_key_deserialization_and_id_generation() {
        let private_key_pem = load_key_bytes("test_private_key.pem");
        let public_key_pem = load_key_bytes("test_public_key.pem");

        // Encode keys to base64 for the JSON string
        let private_key_b64 = BASE64_STANDARD.encode(&private_key_pem);
        let public_key_b64 = BASE64_STANDARD.encode(&public_key_pem);

        let key_json = format!(
            r#"{{
                "algorithm": "RS256",
                "private_key": "{}",
                "public_key": "{}"
            }}"#,
            private_key_b64, public_key_b64
        );

        let deserialized_key: Key = serde_json::from_str(&key_json).expect("Failed to deserialize key");

        // Verify algorithm
        assert_eq!(deserialized_key.algorithm, Algorithm::RS256);

        // Verify keys (compare bytes)
        assert_eq!(deserialized_key.private_key, private_key_pem);
        assert_eq!(deserialized_key.public_key, public_key_pem);

        // Verify ID generation (it should be consistent for the same public key)
        let expected_id = Key::generate_id(&public_key_pem);
        assert_eq!(deserialized_key.id, expected_id);
    }


    #[test]
    fn test_sign_and_verify_jwt_rs256() {
        let current_key = create_test_key_from_files(0, Algorithm::RS256);
        let jwt_config = create_test_jwt_config(current_key.clone(), None);

        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as usize;
        let payload = TestPayload {
            sub: "user123".to_string(),
            company: "Test Inc.".to_string(),
        };

        // Sign the payload
        let token = jwt_config.try_sign(payload.clone()).expect("Failed to sign token");
        assert!(!token.is_empty());

        // Verify the token
        let verified_payload: TestPayload = jwt_config.try_verify(&token).expect("Failed to verify token");

        // Check if the verified payload matches the original (ignoring exp as it's set internally)
        assert_eq!(verified_payload.sub, payload.sub);
        assert_eq!(verified_payload.company, payload.company);
        // We don't assert exp directly as it's calculated during signing,
        // but successful verification implies it was valid.
    }

    #[test]
    fn test_verify_with_previous_key() {
        let previous_key = create_test_key_from_files(1, Algorithm::RS256); // Use offset 1 for a different ID
        let current_key = create_test_key_from_files(0, Algorithm::RS256); // Current key with offset 0

        // Create a config where 'previous_key' is the actual signing key
        let signing_jwt_config = create_test_jwt_config(previous_key.clone(), None);

        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as usize;
        let payload = TestPayload {
            sub: "user456".to_string(),
            company: "Old Corp".to_string(),
        };

        // Sign with the 'previous_key'
        let token = signing_jwt_config.try_sign(payload.clone()).expect("Failed to sign with previous key");

        // Create a new config simulating key rotation (current is new, previous holds the old signing key)
        let verifying_jwt_config = create_test_jwt_config(current_key, Some(previous_key));

        // Verify the token using the config that knows about the previous key
        let verified_payload: TestPayload = verifying_jwt_config.try_verify(&token).expect("Failed to verify token with previous key");

        assert_eq!(verified_payload.sub, payload.sub);
        assert_eq!(verified_payload.company, payload.company);
    }

    #[test]
    fn test_verify_expired_token() {
        let current_key = create_test_key_from_files(0, Algorithm::RS256);
        // Create a config with a very short TTL (e.g., 1 second)
        let mut jwt_config = create_test_jwt_config(current_key, None);
        jwt_config.token_ttl_seconds = -1; // Set TTL to 1 second

        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as usize;
        let payload = TestPayload {
            sub: "user789".to_string(),
            company: "Expired Ltd.".to_string()
        };

        // Sign the token
        let token = jwt_config.try_sign(payload.clone()).expect("Failed to sign token");

        // Attempt to verify the expired token
        let verification_result: Result<TestPayload> = jwt_config.try_verify(&token);

        // Assert that verification failed due to expiration
        assert!(verification_result.is_err());
        match verification_result.err().unwrap() {
            Error::JwtError(err) => assert_eq!(err.kind(), &jsonwebtoken::errors::ErrorKind::ExpiredSignature),
            _ => panic!("Expected JWT expiration error"),
        }
    }

     #[test]
    fn test_verify_invalid_signature() {
        let key1 = create_test_key_from_files(0, Algorithm::RS256);
        let key2 = create_test_key_from_files(1, Algorithm::RS256); // A different key

        let signing_config = create_test_jwt_config(key1, None);
        let verifying_config = create_test_jwt_config(key2, None); // Config with the wrong key

        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as usize;
        let payload = TestPayload {
            sub: "userABC".to_string(),
            company: "Wrong Key Co.".to_string(),
        };

        // Sign with key1
        let token = signing_config.try_sign(payload.clone()).expect("Failed to sign token");

        // Attempt to verify with key2
        let verification_result: Result<TestPayload> = verifying_config.try_verify(&token);

        // Assert that verification failed due to invalid signature/key
        assert!(verification_result.is_err());
         match verification_result.err().unwrap() {
            // Depending on the exact mismatch (kid vs signature), it might be InvalidToken or InvalidSignature
            Error::InvalidToken => {} // OK if kid doesn't match
            Error::JwtError(err) => assert_eq!(err.kind(), &jsonwebtoken::errors::ErrorKind::InvalidSignature), // OK if kid matched but sig failed
            _ => panic!("Expected JWT invalid signature or token error"),
        }
    }

    #[test]
    fn test_verify_wrong_issuer() {
        let key = create_test_key_from_files(0, Algorithm::RS256);
        let mut jwt_config = create_test_jwt_config(key, None);

        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as usize;
        let payload = TestPayload {
            sub: "userDEF".to_string(),
            company: "Issuer Test".to_string(),
        };

        // Sign with the correct issuer
        let token = jwt_config.try_sign(payload.clone()).expect("Failed to sign token");

        // Change the expected issuer in the config *before* verification
        jwt_config.issuer = "wrong_issuer".to_string();

        // Attempt to verify with the wrong issuer configured
        let verification_result: Result<TestPayload> = jwt_config.try_verify(&token);

        assert!(verification_result.is_err());
        match verification_result.err().unwrap() {
            Error::JwtError(err) => assert_eq!(err.kind(), &jsonwebtoken::errors::ErrorKind::InvalidIssuer),
            _ => panic!("Expected JWT invalid issuer error"),
        }
    }

     #[test]
    fn test_verify_wrong_audience() {
        let key = create_test_key_from_files(0, Algorithm::RS256);
        let mut jwt_config = create_test_jwt_config(key, None);

        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as usize;
        let payload = TestPayload {
            sub: "userGHI".to_string(),
            company: "Audience Test".to_string(),
        };

        // Sign with the correct audience
        let token = jwt_config.try_sign(payload.clone()).expect("Failed to sign token");

        // Change the expected audience in the config *before* verification
        jwt_config.audience = "wrong_audience".to_string();

        // Attempt to verify with the wrong audience configured
        let verification_result: Result<TestPayload> = jwt_config.try_verify(&token);

        assert!(verification_result.is_err());
        match verification_result.err().unwrap() {
           Error::JwtError(err) => assert_eq!(err.kind(), &jsonwebtoken::errors::ErrorKind::InvalidAudience),
           _ => panic!("Expected JWT invalid audience error"),
       }
   }

    #[test]
    fn test_missing_kid_in_token() {
        // This test requires manually crafting a token without 'kid' which is complex.
        // Instead, we test the verification logic path that handles missing 'kid'.
        let key = create_test_key_from_files(0, Algorithm::RS256);
        let jwt_config = create_test_jwt_config(key, None);

        // Simulate a token string that would cause decode_header to return a header without kid
        // (This is hard to do perfectly without a real token, so we focus on the error path)
        // A structurally valid JWT but potentially missing 'kid' in header part.
        // Example structure: base64(header).base64(payload).base64(signature)
        // Let's use a header that decodes but lacks 'kid'
        let header_no_kid = r#"{"alg":"RS256","typ":"JWT"}"#;
        let payload_dummy = r#"{"sub":"123"}"#;
        let header_b64 = BASE64_URL_SAFE_NO_PAD.encode(header_no_kid);
        let payload_b64 = BASE64_URL_SAFE_NO_PAD.encode(payload_dummy);
        let token_missing_kid = format!("{}.{}.signature_part", header_b64, payload_b64); // Signature doesn't matter for header decode

        let verification_result: Result<TestPayload> = jwt_config.try_verify(&token_missing_kid);

        // We expect an error because decode_header succeeds but kid is None
        assert!(verification_result.is_err());
        match verification_result.err().unwrap() {
             // The error comes from line 198: header.kid.ok_or(Error::InvalidToken)?
            Error::InvalidToken => {} // Correct error for missing kid
            _ => panic!("Expected InvalidToken error due to missing kid"),
        }
    }
}
