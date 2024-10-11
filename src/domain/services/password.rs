use argon2::{password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString}, Argon2};
use crate::domain::types::Result;
use static_init::dynamic;


#[dynamic]
static ARGON: Argon2<'static> = Argon2::default();


pub struct Password;


impl Password {
    pub fn hash(password: &str) -> Result<String> {
        let salt = SaltString::generate(OsRng);
        let hash = ARGON.hash_password(password.as_bytes(), &salt)?.to_string();
        Ok(hash)
    }
    pub fn verify(password: &str, hash: &str) -> Result<()> {
        let hash = PasswordHash::new(hash)?;
        Ok(ARGON.verify_password(password.as_bytes(), &hash)?)
    }
}