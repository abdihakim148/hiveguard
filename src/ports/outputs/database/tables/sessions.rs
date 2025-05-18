use crate::types::{Error, Id, Session};

pub trait SessionsTable<Client> {
    type Error: Into<Error>;
    async fn create_session(&self, session: Session, client: &Client) -> Result<(), Self::Error>;
    async fn get_session_by_id(&self, id: Id, client: &Client) -> Result<Option<Session>, Self::Error>;
    async fn get_sessions_by_user_id(&self, user_id: Id, client: &Client) -> Result<Vec<Session>, Self::Error>;
    async fn change_current_refresh_token(
        &self,
        id: Id,
        new_refresh_token_id: Id,
        previous_refresh_token_id: Id,
        client: &Client
    ) -> Result<Session, Self::Error>;
    async fn delete_session(&self, id: Id, client: &Client) -> Result<(), Self::Error>;
}
