pub trait UserRepository {
    fn create_user(&self, user: &User) -> Result<(), String>;
    fn get_user(&self, user_id: &str) -> Option<User>;
    fn update_user(&self, user: &User) -> Result<(), String>;
    fn delete_user(&self, user_id: &str) -> Result<(), String>;
}

pub trait AuthenticationService {
    fn login(&self, username: &str, password: &str) -> Result<String, String>;
    fn logout(&self, token: &str) -> Result<(), String>;
}

pub struct User {
    pub id: String,
    pub username: String,
    pub password_hash: String,
    // Add more fields as needed
}
