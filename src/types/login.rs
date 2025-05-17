use serde::{Serialize, Deserialize};
use super::OAuthProvider;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum Login {
    #[serde(rename = "password")]
    Password(String),
    #[serde(rename = "oauth")]
    OAuth(OAuthProvider),
}

impl Login {
    // pub fn clear_password(&mut self) {
    //     if let Login::Password(ref mut password) = self {
    //         *password = String::new();
    //     }
    // }

    pub fn is_empty(&self) -> bool {
        match self {
            Login::Password(ref password) => password.is_empty(),
            Login::OAuth(_) => false,
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_login_serialization() {
        let login = Login::Password("password123".to_string());
        let serialized = serde_json::to_string(&login).unwrap();
        assert_eq!(serialized, r#"{"password":"password123"}"#);
    }

    #[test]
    fn test_login_deserialization() {
        let data = json!({"oauth": "github"});
        let deserialized: Login = serde_json::from_value(data).unwrap();
        assert_eq!(deserialized, Login::OAuth(OAuthProvider::Github));
    }
}