use crate::types::{User, Id, Email, Phone, Error};
use serde_json::{Map, Value};

pub trait UsersTable<Client> {
    type Error: Into<Error>;
    async fn create_user(&self, user: User, client: &Client) -> Result<(), Self::Error>;
    async fn get_user_by_id(&self, id: Id, client: &Client) -> Result<Option<User>, Self::Error>;
    async fn get_user_by_email(&self, email: Email, client: &Client) -> Result<Option<User>, Self::Error>;
    async fn get_user_by_phone(&self, phone: Phone, client: &Client) -> Result<Option<User>, Self::Error>;
    async fn update_user(&self, id: Id, update: Map<String, Value>, client: &Client) -> Result<User, Self::Error>;
    async fn delete_user(&self, id: Id, client: &Client) -> Result<(), Self::Error>;
}