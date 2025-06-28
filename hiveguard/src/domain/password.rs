use password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString};
use crate::types::Error;


pub trait Password {
    fn hash_password(&self, password: &str) -> Result<String, Error>;
    fn verify_password(&self, password: &str, hash: &str) -> Result<(), Error>;
}


impl<T: PasswordHasher + PasswordVerifier> Password for T {
    fn hash_password(&self, password: &str) -> Result<String, Error> {
        let salt = SaltString::generate(&mut OsRng);
        let password_hash = self.hash_password(password.as_bytes(), &salt)?;
        Ok(password_hash.to_string())
    }

    fn verify_password(&self, password: &str, hash: &str) -> Result<(), Error> {
        let parsed_hash = PasswordHash::new(hash)?;
        self.verify_password(password.as_bytes(), &parsed_hash)?;
        Ok(())
    }
}