use crate::ports::outputs::database::{Database, tables::SessionsTable};
use crate::types::{Token, TokenBundle, Id, Session};


pub trait Tokenizer {
    type Error;
    async fn generate_token<DB: Database<SessionsTable: SessionsTable<DB::Client, Item = Session>>>(&self, db: &DB, subject: Id) -> Result<TokenBundle, Self::Error> where Self::Error: From<DB::Error>;
    async fn renew_token<DB: Database<SessionsTable: SessionsTable<DB::Client, Item = Session>>>(&self, db: &DB, token: &Token) -> Result<Token, Self::Error> where Self::Error: From<DB::Error>;
    async fn renew_refresh_token<DB: Database<SessionsTable: SessionsTable<DB::Client, Item = Session>>>(&self, db: &DB, token: &Token) -> Result<Token, Self::Error> where Self::Error: From<DB::Error>;
    async fn invalidate_token<DB: Database<SessionsTable: SessionsTable<DB::Client, Item = Session>>>(&self, db: &DB, token: &Token) -> Result<(), Self::Error> where Self::Error: From<DB::Error>;
    async fn validate_token(&self, token: &Token) -> Result<(), Self::Error>;
}