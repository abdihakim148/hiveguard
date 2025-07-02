pub mod tables;

use crate::types::{Email, Id, Phone};
use macros::{client, database};
use serde_json::{Map, Value};
use tables::*;

#[database]
pub trait Database {
    type Client;
    type Error;
    type UsersTable: UsersTable<Self::Client, Error: Into<Self::Error>>;
    type SessionsTable: SessionsTable<Self::Client, Error: Into<Self::Error>>;
    type VerificationsTable: VerificationsTable<Self::Client, Error: Into<Self::Error>>;

    fn users_table(&self) -> &Self::UsersTable;
    fn sessions_table(&self) -> &Self::SessionsTable;
    fn verifications_table(&self) -> &Self::VerificationsTable;
    #[client]
    fn client(&self) -> &Self::Client;
}
