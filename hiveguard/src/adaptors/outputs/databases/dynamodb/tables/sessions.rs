use aws_sdk_dynamodb::{types::{AttributeValue, KeysAndAttributes}, Client, error::BuildError};
use crate::ports::outputs::database::tables::SessionsTable as Table;
use crate::types::{Session, Id, DatabaseError};

pub struct SessionsTable{
    pub name: String,
}


impl SessionsTable {
    fn keys_and_attributes(key: String, value: AttributeValue) -> Result<KeysAndAttributes, BuildError> {
        let input = [(key, value)].into();
        KeysAndAttributes::builder().keys(input).build()
    }
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
        let (key, value) = ("user_id".into(), user_id.into());
        let keys = Self::keys_and_attributes(key, value)?;
        let output = client.batch_get_item().request_items(&self.name, keys).send().await?;
        match output.responses {
            Some(mut tables) => {
                let mut sessions = vec![];
                if let Some(items) = tables.remove(&self.name) {
                    for item in items {
                        sessions.push(item.try_into()?);
                    }
                }
                Ok(sessions)
            },
            None => Ok(vec![])
        }
    }

    async fn change_current_refresh_token(
        &self,
        id: Id,
        new_refresh_token_id: Id,
        client: &Client
    ) -> Result<(), Self::Error> {
        let (k, v) = ("id", id.into());
        let update_expression = "SET previous_refresh_token_id = refresh_token_id, refresh_token_id = :new_id";
        let (key, value) = (":new_id", new_refresh_token_id.into());
        let _ = client.update_item()
            .table_name(&self.name)
            .key(k, v)
            .update_expression(update_expression)
            .expression_attribute_values(key, value)
            .send()
            .await?;
        Ok(())
    }

    async fn delete_session(&self, id: Id, client: &Client) -> Result<(), Self::Error> {
        let (k, v) = ("id", id.into());
        client.delete_item().table_name(&self.name).key(k, v).send().await?;
        Ok(())
    }
}