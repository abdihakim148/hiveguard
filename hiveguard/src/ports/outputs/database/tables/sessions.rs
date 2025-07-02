use macros::{table, skip};
use crate::types::Id;


#[table]
pub trait SessionsTable<Client> {
    type Error;
    type Item;
    #[skip(Error)]
    async fn create_session(&self, session: Self::Item, client: &Client) -> Result<(), Self::Error>;
    #[skip(Error)]
    async fn get_session_by_id(&self, id: Id, client: &Client) -> Result<Option<Self::Item>, Self::Error>;
    #[skip(Error)]
    async fn get_sessions_by_user_id(&self, user_id: Id, client: &Client) -> Result<Vec<Self::Item>, Self::Error>;
    #[skip(Error)]
    async fn change_current_refresh_token(
        &self,
        id: Id,
        new_refresh_token_id: Id,
        client: &Client
    ) -> Result<(), Self::Error>;
    #[skip(Error)]
    async fn delete_session(&self, id: Id, client: &Client) -> Result<(), Self::Error>;
}
