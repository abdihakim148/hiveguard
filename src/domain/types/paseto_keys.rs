use rand::{rngs::OsRng, CryptoRng, TryRngCore}; // Importing OsRng for generating random numbers securely.
use ed25519_dalek::{SigningKey, SecretKey}; // Importing the SigningKey for generating key pairs.
use serde::{Serialize, Deserialize}; // Importing necessary traits for serialization and deserialization.
use chrono::{DateTime, Utc}; // Importing DateTime and Utc for handling time-related data.
use chrono::Duration; // Importing Duration for handling time intervals.
use std::ops::DerefMut;


/// The duration for which the PASETO keys are valid.
const PASETO_KEYS_DURATION: Duration = Duration::days(90);

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)] // Deriving traits for PasetoKeys struct.
/// A struct representing the PASETO keys.
///
/// This struct holds the private and public keys used for PASETO operations,
/// along with the previous public key, creation time, and expiration time.
pub struct PasetoKeys {
    pub private_key: [u8; 32], // The private key used for signing.
    pub public_key: [u8; 32], // The public key used for verification.
    pub prev_public_key: Option<[u8; 32]>, // The previous public key, if any, for backward compatibility.
    pub created_time: DateTime<Utc>, // The time when the keys were created.
    pub expires: DateTime<Utc> // The expiration time of the keys.
}


impl PasetoKeys {
    fn generate() -> SigningKey {
        let mut csprng = OsRng;
        let mut secret = [0u8; 32];
        csprng.try_fill_bytes(&mut secret).expect("Failed to generate random bytes");
        SigningKey::from_bytes(&secret)
    }
}



impl Default for PasetoKeys {
    /// Generates a default set of PASETO keys.
    ///
    /// This implementation generates a new key pair using a secure random number generator,
    /// sets the creation time to the current time, and calculates the expiration time based
    /// on the defined duration.
    fn default() -> Self {
        // Generate a new signing key using a secure random number generator.
        let key = Self::generate();
        
        // Extract the public key from the signing key.
        let public_key = key.verifying_key().to_bytes();
        
        // Convert the signing key to a byte array for the private key.
        let private_key = key.to_bytes();
        
        let prev_public_key = None; // Initialize the previous public key as None.
        
        let created_time = Utc::now(); // Set the creation time to the current time.
        
        let expires = created_time + PASETO_KEYS_DURATION; // Calculate the expiration time.
        
        // Return a new instance of PasetoKeys with the generated values.
        Self {private_key, public_key, prev_public_key, created_time, expires}
    }
}
