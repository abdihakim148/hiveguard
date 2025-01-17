use argon2::{password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString}, Argon2};
use super::super::types::Error;
use super::types::ARGON2;
use static_init::dynamic;


type Result<T> = std::result::Result<T, Error>;




pub struct Password;


impl Password {
    pub fn hash(password: &str) -> Result<String> {
        let salt = SaltString::generate(OsRng);
        let hash = ARGON2.read().hash_password(password.as_bytes(), &salt)?.to_string();
        Ok(hash)
    }
    pub fn verify(password: &str, hash: &str) -> Result<()> {
        let hash = PasswordHash::new(hash)?;
        Ok(ARGON2.read().verify_password(password.as_bytes(), &hash)?)
    }
}