use crate::ports::outputs::database::Database;
use crate::types::{Error, User, TokenBundle, Email, DatabaseError};
use super::{Password, Tokenizer};


pub struct Authentication;


impl Authentication {
    pub async fn signup<DB: Database, T: Tokenizer, Hasher: Password>(db: &DB, mut user: User, tokenizer: &T, hasher: Hasher) -> Result<TokenBundle, Error>
    where
        Error: From<DB::Error>,
    {
        let password = user.login.password().ok_or(Error::InvalidCredentials)?;
        let hash = hasher.hash_password(password)?;
        user.login.set_hash(hash);
        let subject = user.id;
        db.create_user(user).await?;
        tokenizer.generate_token(db, subject).await
    }

    pub async fn login<DB: Database, T: Tokenizer, Verifyer: Password>(db: &DB, email: Email, password: String, tokenizer: &T, verifyer: Verifyer) -> Result<TokenBundle, Error> 
    where
        Error: From<DB::Error>,
    {
        let user = match db.get_user_by_email(email).await?{
            Some(user) => user,
            None => return Err(Error::DatabaseError(DatabaseError::UserNotFound)),
        };
        let hash = user.login.password().ok_or(Error::InvalidCredentials)?;
        verifyer.verify_password(&password, hash)?;
        let subject = user.id;
        tokenizer.generate_token(db, subject).await
    }
}