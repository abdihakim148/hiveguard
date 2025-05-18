use crate::ports::outputs::database::Database;
use aws_sdk_dynamodb::Client;
use tables::*;

pub mod tables;


pub struct DynamoDB {
    client: Client,
    users_table: UsersTable,
    sessions_table: SessionsTable,
    verifications_table: VerificationsTable,
}

impl Database for DynamoDB {
    type Client = Client;
    type SessionsTable = SessionsTable;
    type UsersTable = UsersTable;
    type VerificationsTable = VerificationsTable;
    
    fn users_table(&self) -> &tables::UsersTable {
        &self.users_table
    }

    fn sessions_table(&self) -> &tables::SessionsTable {
        &self.sessions_table
    }

    fn verifications_table(&self) -> &tables::VerificationsTable {
        &self.verifications_table
    }
}