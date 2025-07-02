use crate::ports::outputs::database::tables::UsersTable as Table;
use aws_sdk_dynamodb::types::{AttributeValue, ReturnValue};
use crate::types::{User, Id, DatabaseError, Phone, Email};
use aws_sdk_dynamodb::Client;
use serde_json::{Map, Value};
use super::map_to_hash_map;


pub struct UsersTable{
    pub name: String
}

impl Table<Client> for UsersTable {
    type Error = DatabaseError;
    type Item = User;
    async fn create_user(&self, user: Self::Item, client: &Client) -> Result<(), Self::Error> {
        let input = Some(user.into());
        let _ = client.put_item().table_name(&self.name).set_item(input).send().await?;
        Ok(())
    }

    async fn get_user_by_id(&self, id: Id, client: &Client) -> Result<Option<Self::Item>, Self::Error> {
        let (k, v) = ("id", id.into());
        let output = client.get_item().table_name(&self.name).key(k, v).send().await?;
        match output.item {
            Some(item) => Ok(Some(item.try_into()?)),
            None => Ok(None)
        }
    }

    async fn get_user_by_email(&self, email: Email, client: &Client) -> Result<Option<Self::Item>, Self::Error> {
        let (k, v) = ("email", AttributeValue::S(email.to_string()));
        let output = client.get_item().table_name(&self.name).key(k, v).send().await?;
        match output.item {
            Some(item) => Ok(Some(item.try_into()?)),
            None => Ok(None)
        }
    }

    async fn get_user_by_phone(&self, phone: Phone, client: &Client) -> Result<Option<Self::Item>, Self::Error> {
        let (k, v) = ("phone", AttributeValue::S(phone.to_string()));
        let output = client.get_item().table_name(&self.name).key(k, v).send().await?;
        match output.item {
            Some(item) => Ok(Some(item.try_into()?)),
            None => Ok(None)
        }
    }

    async fn update_user(&self, id: Id, update: Map<String, Value>, client: &Client) -> Result<Self::Item, Self::Error> {
        let (k, v) = ("id", id.into());
        if update.is_empty() {
            let output = client.get_item().table_name(&self.name).key(k, v).send().await?;
            match output.item {
                Some(item) => return Ok(item.try_into()?),
                None => return Err(DatabaseError::UserNotFound)
            }
        }
        let map = map_to_hash_map(update)?;
        let mut builder = client.update_item().table_name(&self.name).key(k, v);
        for (k, v) in map {
            builder = builder.update_expression(format!("SET {} = :{}", k, k));
            builder = builder.expression_attribute_values(format!(":{}", k), v);
        }
        let output = builder.return_values(ReturnValue::AllNew).send().await?;
        match output.attributes {
            Some(item) => return Ok(item.try_into()?),
            None => return Err(DatabaseError::UserNotFound)
        }
    }

    async fn delete_user(&self, id: Id, client: &Client) -> Result<(), Self::Error> {
        let (k, v) = ("id", id.into());
        client.delete_item().table_name(&self.name).key(k, v).send().await?;
        Ok(())
    }
}