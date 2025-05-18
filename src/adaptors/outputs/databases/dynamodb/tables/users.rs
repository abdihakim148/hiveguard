use crate::ports::outputs::database::tables::UsersTable as Table;
use crate::types::{User, Id, DatabaseError, Phone, Email};
use aws_sdk_dynamodb::types::AttributeValue;
use aws_sdk_dynamodb::Client;
use serde_json::{Map, Value};


pub struct UsersTable{
    pub name: String
}

impl Table<Client> for UsersTable {
    type Error = DatabaseError;
    async fn create_user(&self, user: User, client: &Client) -> Result<(), Self::Error> {
        let input = Some(user.into());
        let _ = client.put_item().table_name(&self.name).set_item(input).send().await?;
        Ok(())
    }

    async fn get_user_by_id(&self, id: Id, client: &Client) -> Result<Option<User>, Self::Error> {
        let (k, v) = ("id", id.into());
        let output = client.get_item().table_name(&self.name).key(k, v).send().await?;
        match output.item {
            Some(item) => Ok(Some(item.try_into()?)),
            None => Ok(None)
        }
    }

    async fn get_user_by_email(&self, email: Email, client: &Client) -> Result<Option<User>, Self::Error> {
        let (k, v) = ("email", AttributeValue::S(email.to_string()));
        let output = client.get_item().table_name(&self.name).key(k, v).send().await?;
        match output.item {
            Some(item) => Ok(Some(item.try_into()?)),
            None => Ok(None)
        }
    }

    async fn get_user_by_phone(&self, phone: Phone, client: &Client) -> Result<Option<User>, Self::Error> {
        let (k, v) = ("phone", AttributeValue::S(phone.to_string()));
        let output = client.get_item().table_name(&self.name).key(k, v).send().await?;
        match output.item {
            Some(item) => Ok(Some(item.try_into()?)),
            None => Ok(None)
        }
    }

    async fn update_user(&self, id: Id, update: Map<String, Value>, client: &Client) -> Result<User, Self::Error> {
        todo!("not yet implemented")
    }

    async fn delete_user(&self, id: Id, client: &Client) -> Result<(), Self::Error> {
        todo!("not yet implemented")
    }
}