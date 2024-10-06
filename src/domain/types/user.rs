use serde::{Deserialize, Serialize};
use crate::domain::services::Crud;
use bson::oid::ObjectId;

/// A struct representing a user.
#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq)]
pub struct User {
    /// The unique identifier for the user.
    pub id: ObjectId,
    /// The username of the user.
    pub username: String,
    /// The first name of the user.
    pub first_name: String,
    /// The last name of the user.
    pub last_name: String,
    /// The email address of the user.
    pub email: String,
    /// The password of the user.
    pub password: String,
}



impl Crud for User {
    type Id = ObjectId;
}

#[cfg(test)]
mod tests {
    use super::User;
    use crate::adaptors::database::memory::tables::Users;
    use crate::domain::services::Crud;
    use bson::oid::ObjectId;
    use tokio;

    #[tokio::test]
    async fn test_crud_create_user() {
        let users = Users::new().await.unwrap();
        let user = User {
            id: ObjectId::new(),
            username: "testuser".to_string(),
            first_name: "Test".to_string(),
            last_name: "User".to_string(),
            email: "test@example.com".to_string(),
            password: "password".to_string(),
        };

        let result = user.create(&users).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_crud_read_user() {
        let users = Users::new().await.unwrap();
        let user = User {
            id: ObjectId::new(),
            username: "testuser".to_string(),
            first_name: "Test".to_string(),
            last_name: "User".to_string(),
            email: "test@example.com".to_string(),
            password: "password".to_string(),
        };

        let id = user.create(&users).await.unwrap();
        let read_user = User::read(&users, &id).await.unwrap();
        assert_eq!(Some(user), read_user);
    }

    #[tokio::test]
    async fn test_crud_update_user() {
        let users = Users::new().await.unwrap();
        let mut user = User {
            id: ObjectId::new(),
            username: "testuser".to_string(),
            first_name: "Test".to_string(),
            last_name: "User".to_string(),
            email: "test@example.com".to_string(),
            password: "password".to_string(),
        };

        let id = user.create(&users).await.unwrap();
        user.email = "newemail@example.com".to_string();
        let update_result = user.update(&users).await;
        assert!(update_result.is_ok());

        let updated_user = User::read(&users, &id).await.unwrap();
        assert_eq!(Some(user), updated_user);
    }

    #[tokio::test]
    async fn test_crud_delete_user() {
        let users = Users::new().await.unwrap();
        let user = User {
            id: ObjectId::new(),
            username: "testuser".to_string(),
            first_name: "Test".to_string(),
            last_name: "User".to_string(),
            email: "test@example.com".to_string(),
            password: "password".to_string(),
        };

        let id = user.create(&users).await.unwrap();
        let delete_result = User::delete(&users, &id).await;
        assert!(delete_result.is_ok());

        let deleted_user = User::read(&users, &id).await.unwrap();
        assert!(deleted_user.is_none());
    }
}
}
