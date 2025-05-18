use crate::ports::outputs::database::tables::SessionsTable as Table;
use crate::types::{Session, Id, DatabaseError};
use aws_sdk_dynamodb::Client;

pub struct SessionsTable{
    pub name: String,
}


impl Table<Client> for SessionsTable {
    type Error = DatabaseError;
    async fn create_session(&self, session: Session, client: &Client) -> Result<(), Self::Error> {
        let input = Some(session.into());
        let _ = client.put_item().table_name(&self.name).set_item(input).send().await?;
        Ok(())
    }

    async fn get_session_by_id(&self, id: Id, client: &Client) -> Result<Option<Session>, Self::Error> {
        let (k, v) = ("id", id.into());
        let output = client.get_item().table_name(&self.name).key(k, v).send().await?;
        match output.item {
            Some(item) => Ok(Some(item.try_into()?)),
            None => Ok(None)
        }
    }

    async fn get_sessions_by_user_id(&self, user_id: Id, client: &Client) -> Result<Vec<Session>, Self::Error> {
        todo!("not yet implemented")
    }

    async fn change_current_refresh_token(
        &self,
        id: Id,
        new_refresh_token_id: Id,
        previous_refresh_token_id: Id,
        client: &Client
    ) -> Result<Session, Self::Error> {
        todo!("not yet implemented")
    }

    async fn delete_session(&self, id: Id, client: &Client) -> Result<(), Self::Error> {
        todo!("not yet implemented")
    }
    
}