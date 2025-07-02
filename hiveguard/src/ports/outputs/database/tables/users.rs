use crate::types::{Id, Email, Phone};
use serde_json::{Map, Value};
use macros::{table, skip};

#[table]
pub trait UsersTable<Client> {
    type Error;
    type Item;
    #[skip(Error)]
    async fn create_user(&self, user: Self::Item, client: &Client) -> Result<(), Self::Error>;
    #[skip(Error)]
    async fn get_user_by_id(&self, id: Id, client: &Client) -> Result<Option<Self::Item>, Self::Error>;
    #[skip(Error)]
    async fn get_user_by_email(&self, email: Email, client: &Client) -> Result<Option<Self::Item>, Self::Error>;
    #[skip(Error)]
    async fn get_user_by_phone(&self, phone: Phone, client: &Client) -> Result<Option<Self::Item>, Self::Error>;
    #[skip(Error)]
    async fn update_user(&self, id: Id, update: Map<String, Value>, client: &Client) -> Result<Self::Item, Self::Error>;
    #[skip(Error)]
    async fn delete_user(&self, id: Id, client: &Client) -> Result<(), Self::Error>;
}