use serde::{Serialize, de::DeserializeOwned};
use crate::domain::types::Error;


pub type Result<T> = std::result::Result<T, Error>;

/// Generic trait for token signing and verification operations.
pub trait Tokenizer<Payload>
where
    Payload: Serialize + DeserializeOwned + Send + Sync + 'static, // Payload constraints
{

    /// Signs the given payload using the provided keys and configuration.
    fn try_sign<Input: Into<Payload>>(&self, input: Input) -> Result<String>;

    /// Verifies the given token string using the provided keys and configuration,
    /// returning the deserialized payload if successful.
    /// Note: This verifies structure and signature. Semantic checks (like expiration)
    /// should be performed on the returned payload by the caller.
    fn try_verify(&self, token_str: &str) -> Result<Payload>;
}
