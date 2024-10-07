#![allow(unused)]
use crate::domain::types::Result;

/**
 * A trait representing authentication operations.
 *
 * This trait provides methods for registering, authenticating, deauthenticating,
 * verifying, updating credentials, resetting credentials, and retrieving entity details.
 */
pub trait Authentication: Sized {
    type Id;
    type Value;
    type Token;

    /// Registers a new entity and returns its unique identifier.
    ///
    /// # Returns
    ///
    /// * `Result<Self::Id>` - Returns the unique identifier of the registered entity wrapped in a `Result`.
    async fn register(&self) -> Result<Self::Id>;

    /// Authenticates an entity using the provided value and returns a token.
    ///
    /// # Arguments
    ///
    /// * `value` - A reference to the value used for authentication.
    ///
    /// # Returns
    ///
    /// * `Result<Self::Token>` - Returns the authentication token wrapped in a `Result`.
    async fn authenticate(&self, value: &Self::Value) -> Result<Self::Token>;

    /// Deauthenticates an entity using the provided token.
    ///
    /// # Arguments
    ///
    /// * `token` - A reference to the token used for deauthentication.
    ///
    /// # Returns
    ///
    /// * `Result<()>` - Returns an empty result indicating success or failure.
    async fn deauthenticate(&self, token: &Self::Token) -> Result<()>;

    /// Verifies the validity of the provided token.
    ///
    /// # Arguments
    ///
    /// * `token` - A reference to the token to be verified.
    ///
    /// # Returns
    ///
    /// * `Result<bool>` - Returns `true` if the token is valid, `false` otherwise, wrapped in a `Result`.
    async fn verify(&self, token: &Self::Token) -> Result<bool>;

    /// Updates the credentials of an entity.
    ///
    /// # Arguments
    ///
    /// * `entity_id` - A reference to the unique identifier of the entity.
    /// * `new_value` - A reference to the new value for the credentials.
    ///
    /// # Returns
    ///
    /// * `Result<()>` - Returns an empty result indicating success or failure.
    async fn update_credentials(&self, entity_id: &Self::Id, new_value: &Self::Value) -> Result<()>;

    /// Resets the credentials of an entity identified by a string.
    ///
    /// # Arguments
    ///
    /// * `identifier` - A string slice that holds the identifier for the entity.
    ///
    /// # Returns
    ///
    /// * `Result<()>` - Returns an empty result indicating success or failure.
    async fn reset_credentials(&self, identifier: &str) -> Result<()>;

    /// Retrieves the details of an entity using the provided token.
    ///
    /// # Arguments
    ///
    /// * `token` - A reference to the token used to retrieve entity details.
    ///
    /// # Returns
    ///
    /// * `Result<Self>` - Returns the entity details wrapped in a `Result`.
    async fn get_entity_details(&self, token: &Self::Token) -> Result<Self>;
}
