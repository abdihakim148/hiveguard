pub mod tables;

use crate::types::{User, Email, Phone, Id, Session};
use serde_json::{Map, Value};
use tables::*;


pub trait Database {
    type Client;
    type UsersTable: UsersTable<Self::Client>;
    type SessionsTable: SessionsTable<Self::Client>;
    type VerificationsTable: VerificationsTable<Self::Client>;

    /// These are function written on hand by the developer.
    fn users_table(&self) -> &Self::UsersTable;
    fn sessions_table(&self) -> &Self::SessionsTable;
    fn verifications_table(&self) -> &Self::VerificationsTable;
}