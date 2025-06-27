use crate::ports::outputs::database::Database;
use crate::types::DatabaseError;
use aws_sdk_dynamodb::Client;

pub mod tables;


pub struct DynamoDB {
    client: Client,
    users_table: tables::UsersTable,
    sessions_table: tables::SessionsTable,
    verifications_table: tables::VerificationsTable,
}

impl Database for DynamoDB {
    type Client = Client;
    type Error = DatabaseError;
    type UsersTable = tables::UsersTable;
    type SessionsTable = tables::SessionsTable;
    type VerificationsTable = tables::VerificationsTable;
    
    fn users_table(&self) ->  &Self::UsersTable {
        &self.users_table
    }

    fn sessions_table(&self) ->  &Self::SessionsTable {
        &self.sessions_table
    }

    fn verifications_table(&self) ->  &Self::VerificationsTable {
        &self.verifications_table
    }

    fn client(&self) -> &Self::Client {
        &self.client
    }
}