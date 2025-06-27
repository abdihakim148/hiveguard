use crate::types::{Verification, Id, Email, Phone};
use macros::table;


#[table]
pub trait VerificationsTable<Client> {
    type Error;
    async fn create_verification(&self, verification: Verification<Id>, client: &Client) -> Result<(), Self::Error>;
    async fn get_verification_by_email(&self, email: Email, client: &Client) -> Result<Option<Verification<Id>>, Self::Error>;
    async fn get_verification_by_phone(&self, phone: Phone, client: &Client) -> Result<Option<Verification<Id>>, Self::Error>;
    async fn delete_verification(&self, user_id: Id, client: &Client) -> Result<(), Self::Error>;
}