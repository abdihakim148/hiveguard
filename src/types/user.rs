use serde::{Serialize, Deserialize};
use super::{Login, Contact, Id};
use chrono::{Utc, DateTime};


#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct User {
    pub id: Id,
    pub username: String,
    pub fullname: String,
    #[serde(flatten)]
    pub contact: Contact,
    #[serde(flatten, skip_serializing_if = "Login::is_empty")]
    pub login: Login,
    #[serde(default)]
    pub profile: Option<String>,
    #[serde(default)]
    pub created_at: DateTime<Utc>,
}


#[cfg(test)]
mod tests {
    use super::super::{Email, Phone};
    use super::*;

    #[test]
    fn test_user_serialization_and_deserialization() {
        let id = Id::try_from(String::from("000000000000000000000000")).unwrap();
        let username = String::from("username");
        let fullname = String::from("fullname");
        let email = Email::try_from("user@example.com").unwrap();
        let phone = Phone::try_from(String::from("+25478965439")).unwrap();
        let contact = Contact::Both(phone, email);
        let password = String::from("password");
        let login = Login::Password(password);
        let profile = None;
        let created_at = Utc::now();
        let user = User {
            id,
            username,
            fullname,
            contact,
            login,
            profile,
            created_at,
        };

        let serialized = serde_json::to_string(&user).unwrap();
        println!("Serialized User: {}", serialized);
        let deserialized = serde_json::from_str::<User>(&serialized).unwrap();
        assert_eq!(user, deserialized);
    }
}