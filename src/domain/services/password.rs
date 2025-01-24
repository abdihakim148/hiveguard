use argon2::password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString};
use super::super::types::Error; // Importing custom error type


type Result<T> = std::result::Result<T, Error>; // Alias for Result with custom Error




/**
 * Trait for password operations including hashing and verification.
 */
pub trait Password {
    /// Hashes the password using Argon2.
    ///
    /// # Arguments
    ///
    /// * `argon2` - Reference to Argon2 hasher.
    ///
    /// # Returns
    ///
    /// * `Result<String>` - The hashed password as a string.
    fn hash<T: PasswordHasher>(&self, argon2: &T) -> Result<String>;
    /// Verifies the password against a hash using Argon2.
    ///
    /// # Arguments
    ///
    /// * `hash` - The hash to verify against.
    /// * `argon2` - Reference to Argon2 verifier.
    ///
    /// # Returns
    ///
    /// * `Result<()>` - Ok if the password matches the hash, otherwise an error.
    fn verify<T: PasswordVerifier>(&self, hash: &str, argon2: &T) -> Result<()>;
}


impl Password for &str { // Implementing Password trait for &str
    fn hash<T: PasswordHasher>(&self, argon2: &T) -> Result<String> {
        let salt = SaltString::generate(OsRng); // Generate a random salt
        let hash = argon2.hash_password(self.as_bytes(), &salt)?.to_string(); // Hash the password
        Ok(hash)
    }
    
    fn verify<T: PasswordVerifier>(&self, hash: &str, argon2: &T) -> Result<()> { // Verify password
        let hash = PasswordHash::new(hash)?; // Parse the hash
        Ok(argon2.verify_password(self.as_bytes(), &hash)?) // Verify the password
    }
}

impl Password for &String { // Implementing Password trait for &String
    fn hash<T: PasswordHasher>(&self, argon2: &T) -> Result<String> {
        let salt = SaltString::generate(OsRng); // Generate a random salt
        let hash = argon2.hash_password(self.as_bytes(), &salt)?.to_string(); // Hash the password
        Ok(hash)
    }
    
    fn verify<T: PasswordVerifier>(&self, hash: &str, argon2: &T) -> Result<()> { // Verify password
        let hash = PasswordHash::new(hash)?; // Parse the hash
        Ok(argon2.verify_password(self.as_bytes(), &hash)?) // Verify the password
    }
}

impl Password for String { // Implementing Password trait for String
    fn hash<T: PasswordHasher>(&self, argon2: &T) -> Result<String> {
        let salt = SaltString::generate(OsRng); // Generate a random salt
        let hash = argon2.hash_password(self.as_bytes(), &salt)?.to_string(); // Hash the password
        Ok(hash)
    }
    
    fn verify<T: PasswordVerifier>(&self, hash: &str, argon2: &T) -> Result<()> { // Verify password
        let hash = PasswordHash::new(hash)?; // Parse the hash
        Ok(argon2.verify_password(self.as_bytes(), &hash)?) // Verify the password
    }
}
