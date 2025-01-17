use argon2::{password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString}, Argon2};
use super::super::types::Error;
use super::types::ARGON2;
use static_init::dynamic;


type Result<T> = std::result::Result<T, Error>;




pub struct Password;


impl Password {
    pub fn hash(password: &str, argon2: &Argon2<'_>) -> Result<String> {
        let salt = SaltString::generate(OsRng);
        let hash = argon2.hash_password(password.as_bytes(), &salt)?.to_string();
        Ok(hash)
    }
    pub fn verify(password: &str, hash: &str, argon2: &Argon2<'_>) -> Result<()> {
        let hash = PasswordHash::new(hash)?;
        Ok(argon2.verify_password(password.as_bytes(), &hash)?)
    }
}