use crate::types::{Id, Email, Phone};
use macros::{table, skip};


#[table]
pub trait VerificationsTable<Client> {
    type Error;
    type Item;
    #[skip(Error)]
    async fn create_verification_code(&self, verification_code: Self::Item, client: &Client) -> Result<(), Self::Error>;
    #[skip(Error)]
    async fn get_verification_by_email(&self, email: Email, client: &Client) -> Result<Option<Self::Item>, Self::Error>;
    #[skip(Error)]
    async fn get_verification_by_phone(&self, phone: Phone, client: &Client) -> Result<Option<Self::Item>, Self::Error>;
    #[skip(Error)]
    async fn delete_verification(&self, user_id: Id, client: &Client) -> Result<(), Self::Error>;
}