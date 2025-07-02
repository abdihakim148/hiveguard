use crate::ports::outputs::database::{Database, tables::{UsersTable, VerificationsTable, SessionsTable}};
use crate::types::{Error, User, TokenBundle, Email, DatabaseError, Verification, Id, Session};
use super::{Password, Tokenizer};


pub struct Authentication;


impl Authentication {
    pub async fn signup<DB: Database<UsersTable: UsersTable<DB::Client, Item = User>, VerificationsTable: VerificationsTable<DB::Client, Item = Verification<Id>>, SessionsTable: SessionsTable<DB::Client, Item = Session>>, T: Tokenizer, Hasher: Password>(db: &DB, mut user: User, tokenizer: &T, hasher: Hasher) -> Result<TokenBundle, Error>
    where
        Error: From<DB::Error>,
        Error: From<T::Error>,
        T::Error: From<DB::Error>
    {
        let password = user.login.password()?;
        let hash = hasher.hash_password(password)?;
        user.login.set_hash(hash);
        let subject = user.id;
        db.create_user(user).await?;
        Ok(tokenizer.generate_token(db, subject).await?)
    }

    pub async fn login<DB: Database<UsersTable: UsersTable<DB::Client, Item = User>, VerificationsTable: VerificationsTable<DB::Client, Item = Verification<Id>>, SessionsTable: SessionsTable<DB::Client, Item = Session>>, T: Tokenizer, Verifyer: Password>(db: &DB, email: Email, password: String, tokenizer: &T, verifyer: Verifyer) -> Result<TokenBundle, Error> 
    where
        Error: From<DB::Error>,
        Error: From<T::Error>,
        T::Error: From<DB::Error>
    {
        let user = match db.get_user_by_email(email).await?{
            Some(user) => user,
            None => return Err(Error::DatabaseError(DatabaseError::UserNotFound)),
        };
        let hash = user.login.password()?;
        verifyer.verify_password(&password, hash)?;
        let subject = user.id;
        Ok(tokenizer.generate_token(db, subject).await?)
    }
}