use chrono::Utc;
use rusty_paseto::core::{ImplicitAssertion, PasetoError, Payload, Public, V4};
use rusty_paseto::core::Paseto as PasetoBuilder;
use serde::{Serialize, de::DeserializeOwned};
use super::super::types::{Error, Token, PasetoKeys, Value};
use rusty_paseto::core::Footer;
use rusty_paseto::core::Key;


type Result<T> = std::result::Result<T, Error>;
/// A trait for handling PASETO (Platform-Agnostic Security Tokens) operations.
///
/// This trait provides methods for signing and verifying PASETO tokens using
/// asymmetric keys. It requires the implementing type to be serializable and
/// deserializable.
///
/// # Methods
///
/// * `expired` - Checks if the token has expired.
/// * `try_verify` - Verifies a PASETO token signature and returns the deserialized token if valid.
/// * `try_sign` - Signs a token and returns the PASETO token string.
pub trait Paseto: Serialize + DeserializeOwned + 'static {
    /// Checks if the token has expired.
    ///
    /// # Returns
    ///
    /// * `bool` - Returns `true` if the token has expired, `false` otherwise.
    fn expired(&self) -> bool;

    /// Verifies a PASETO token signature and returns the deserialized token if valid.
    ///
    /// This method attempts to verify the provided PASETO token signature using the
    /// current public key. If the signature is invalid, it attempts verification
    /// with a previous public key if available.
    ///
    /// # Arguments
    ///
    /// * `signature` - A string slice that holds the PASETO token signature.
    /// * `keys` - A reference to `PasetoKeys` containing the public keys.
    ///
    /// # Returns
    ///
    /// * `Result<Self>` - Returns the deserialized token if the signature is valid, or an error if not.
    fn try_verify(signature: &str, keys: &PasetoKeys) -> Result<Self> {
        // Convert the current public key to the required format.
        let key = Key::from(&keys.public_key);
        
        // Create a public key from the converted key.
        let public_key = From::from(&key);
        
        // Initialize footer and implicit assertion as None.
        let footer = Option::<Footer>::None;
        
        let implicit_assertion = Option::<ImplicitAssertion>::None;
        
        // Attempt to verify the signature using the current public key.
        let json = match PasetoBuilder::try_verify(signature, &public_key, footer, implicit_assertion) {
            Ok(value) => value,
            
            // If verification fails, handle the error.
            Err(err) => {
                
                match err {
                    // If the signature is invalid, try using the previous public key if available.
                    PasetoError::InvalidSignature => { 
                        let key = match keys.prev_public_key { 
                            Some(key) => Key::from(key),
                            None => Err(err)?
                        };
                        // Create a public key from the previous key.
                        let public_key = From::from(&key);
                        
                        // Attempt to verify the signature using the previous public key.
                        PasetoBuilder::try_verify(signature, &public_key, footer, implicit_assertion)? 
                    },
                    // Propagate other errors.
                    _ => Err(err)? 
                }
            }
        };
        
        // Deserialize the JSON payload into the expected type.
        match serde_json::from_str(&json) { 
            Ok(value) => Ok(value),
            // Return an error if deserialization fails.
            Err(err) => Err(Error::InvalidToken) 
        }
    }


    /// Signs a token and returns the PASETO token string.
    ///
    /// This method serializes the token and signs it using the provided private key.
    ///
    /// # Arguments
    ///
    /// * `keys` - A reference to `PasetoKeys` containing the private key.
    ///
    /// # Returns
    ///
    /// * `Result<String>` - Returns the PASETO token string if successful, or an error if not.
    fn try_sign(&self, keys: &PasetoKeys) -> Result<String> {
        // Prepare the key by combining the private and public keys.
        let mut key = [0u8; 64];
        
        key[..32].copy_from_slice(&keys.private_key);
        
        key[32..].copy_from_slice(&keys.public_key);
        
        // Convert the combined key to the required format.
        let key = Key::from(&key);
        
        let key = From::from(&key);
        // Serialize the token to a JSON string.
        let json = serde_json::to_string(&self).map_err(|err|Error::<Value>::InternalServerError(err.into()))?;
        
        // Create a payload from the JSON string.
        let payload = Payload::from(json.as_str());
        
        // Sign the payload to generate the PASETO token.
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
