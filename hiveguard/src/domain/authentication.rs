use crate::ports::outputs::database::Database;
use crate::types::{Error, User, TokenBundle, Email, DatabaseError};
use super::{Password, Tokenizer};


pub struct Authentication;


impl Authentication {
    pub async fn signup<DB: Database, T: Tokenizer>(db: &DB, user: User, tokenizer: &T) -> Result<TokenBundle, Error>
    where
        Error: From<DB::Error>,
    {
        let subject = user.id;
        db.create_user(user).await?;
        tokenizer.generate_token(db, subject).await
    }

    pub async fn login<DB: Database, T: Tokenizer>(db: &DB, email: Email, tokenizer: &T) -> Result<TokenBundle, Error> 
    where
        Error: From<DB::Error>,
    {
        let user = match db.get_user_by_email(email).await?{
            Some(user) => user,
            None => return Err(Error::DatabaseError(DatabaseError::UserNotFound)),
        };
        let subject = user.id;
        tokenizer.generate_token(db, subject).await
    }
}