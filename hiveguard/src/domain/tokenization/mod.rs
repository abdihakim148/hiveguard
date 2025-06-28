use crate::types::{Error, Token, TokenBundle, Id};
use crate::ports::outputs::database::Database;


pub trait Tokenizer {
    async fn generate_token<DB: Database>(&self, db: &DB, subject: Id) -> Result<TokenBundle, Error>;
    async fn renew_token<DB: Database>(&self, db: &DB, token: &Token) -> Result<Token, Error>;
    async fn renew_refresh_token<DB: Database>(&self, db: &DB, token: &Token) -> Result<Token, Error>;
    async fn invalidate_token<DB: Database>(&self, db: &DB, token: &Token) -> Result<(), Error>;
    async fn validate_token(&self, token: &Token) -> Result<(), Error>;
}