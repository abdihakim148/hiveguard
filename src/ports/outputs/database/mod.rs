pub mod tables;


use tables::*;


pub trait Database {
    type Client;
    type UsersTable: UsersTable<Self::Client>;
    type SessionsTable: SessionsTable<Self::Client>;
    type VerificationsTable: VerificationsTable<Self::Client>;

    fn users_table(&self) -> &Self::UsersTable;
    fn sessions_table(&self) -> &Self::SessionsTable;
    fn verifications_table(&self) -> &Self::VerificationsTable;
}