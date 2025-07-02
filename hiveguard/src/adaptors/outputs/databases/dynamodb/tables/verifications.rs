use crate::ports::outputs::database::tables::VerificationsTable as Table;
use crate::types::{Verification, Id, DatabaseError};
use aws_sdk_dynamodb::types::AttributeValue;
use aws_sdk_dynamodb::Client;


pub struct VerificationsTable {
    pub name: String,
}


impl Table<Client> for VerificationsTable {
    type Error = DatabaseError;
    type Item = Verification<Id>;
    async fn create_verification_code(&self, verification: Self::Item, client: &Client) -> Result<(), Self::Error> {
        let input = Some(verification.into());
        let _ = client.put_item().table_name(&self.name).set_item(input).send().await?;
        Ok(())
    }

    async fn get_verification_by_email(&self, email: crate::types::Email, client: &Client) -> Result<Option<Self::Item>, Self::Error> {
        let (k, v) = ("email", AttributeValue::S(email.to_string()));
        let output = client.get_item().table_name(&self.name).key(k, v).send().await?;
        match output.item {
            Some(item) => Ok(Some(item.try_into()?)),
            None => Ok(None)
        }
    }

    async fn get_verification_by_phone(&self, phone: crate::types::Phone, client: &Client) -> Result<Option<Self::Item>, Self::Error> {
        let (k, v) = ("phone", AttributeValue::S(phone.to_string()));
        let output = client.get_item().table_name(&self.name).key(k, v).send().await?;
        match output.item {
            Some(item) => Ok(Some(item.try_into()?)),
            None => Ok(None)
        }
    }

    async fn delete_verification(&self, user_id: Id, client: &Client) -> Result<(), Self::Error> {
        let (k, v) = ("user_id", user_id.into());
        client.delete_item().table_name(&self.name).key(k, v).send().await?;
        Ok(())
    }
}