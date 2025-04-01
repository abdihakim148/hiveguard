use serde::{Deserialize, Serialize};
use crate::domain::types::user::User;
use url::Url;

/// OAuth Provider configuration for third-party authentication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Provider {
    /// Client ID from the OAuth provider
    pub client_id: String,
    
    /// Client secret from the OAuth provider
    pub client_secret: String,
    
    /// Authorization endpoint URL
    pub auth_url: Url,
    
    /// Token exchange endpoint URL
    pub token_url: Url,
    
    /// User information endpoint URL
    pub userinfo_url: Url,
    
    /// OAuth scopes required for authentication
    pub scopes: Vec<String>,
    
    /// Mapping of provider's user fields to system's User fields
    /// Must have the same length as User::FIELDS
    pub fields: [Option<String>; User::FIELDS.len()],
}
