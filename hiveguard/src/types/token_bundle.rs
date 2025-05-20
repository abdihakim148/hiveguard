use serde::{Serialize, Deserialize};


#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct TokenBundle {
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: String,
    pub scope: Option<String>,
    pub id_token: Option<String>,
}