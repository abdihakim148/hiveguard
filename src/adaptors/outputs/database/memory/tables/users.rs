use crate::domain::types::{Either, Contact, Key, User, Value};
use crate::ports::outputs::database::{Item, Table}; // Importing necessary traits and types
use crate::domain::types::Error;
use std::collections::HashMap;
use std::sync::RwLock;

/// A struct representing a collection of users stored in memory.
#[derive(Debug, Default)]
pub struct Users {
    primary: RwLock<HashMap<<User as Item>::PK, User>>,
    secondary: RwLock<HashMap<<User as Item>::SK, <User as Item>::PK>>,
}

impl Users {
    /// Checks if a user with the given contact exists.
    ///
    /// # Arguments
    ///
    /// * `contact` - either an contactAddress or a phone number or both
    ///
    /// # Returns
    ///
    /// * `Result<bool>` - Returns `Ok(true)` if the contact exists, `Ok(false)` otherwise.
    async fn exists(&self, contact: &<User as Item>::SK) -> Result<bool, Error> {
        let secondary = self.secondary.read()?;
        match contact {
            Contact::Phone(_) => Ok(secondary.contains_key(contact)),
            Contact::Email(_) => Ok(secondary.contains_key(contact)),
            Contact::Both(phone, contact) => {
                let phone = Contact::Phone(phone.clone());
                let contact = Contact::Email(contact.clone());
                Ok(secondary.contains_key(&phone) || secondary.contains_key(&contact))
            }
        }
    }

    /// Retrieves the contact associated with a given user ID.
    ///
    /// # Arguments
    ///
    /// * `id` - A reference to the ID of the user whose contact is to be retrieved.
    ///
    /// # Returns
    ///
    /// * `Result<Option<String>>` - Returns the contact if found, otherwise `None`, wrapped in a `Result`.
    async fn contact(&self, id: &<User as Item>::PK) -> Result<Option<Contact>, Error> {
        let primary = self.primary.read()?;
        match primary.get(id) {
            None => Ok(None),
            Some(user) => Ok(Some(user.contact.clone())),
        }
    }
}

impl Table for Users {
    type Map = HashMap<String, Value>;
    type Item = User;
    type Error = Error;

    const NAME: &'static str = "users";

    /// Creates a new instance of `Users`.
    ///
    /// # Returns
    ///
    /// * `Result<Self>` - Returns a new `Users` instance wrapped in a `Result`.
    async fn new() -> Result<Self, Self::Error> {
        Ok(Users {
            primary: RwLock::new(HashMap::new()),
            secondary: RwLock::new(HashMap::new()),
        })
    }

    /// Creates a new user.
    ///
    /// # Arguments
    ///
    /// * `user` - A reference to the user item to be created.
    ///
    /// # Returns
    ///
    /// * `Result<<User as Item>::PK>` - Returns the ID of the created user wrapped in a `Result`.
    async fn create(&self, user: &User) -> Result<<User as Item>::PK, Self::Error> {
        if self.exists(&user.contact).await? {
            return Err(Error::Conflict(Self::Item::NAME));
        }

        if let Some(existing_contact) = self.contact(&user.id).await? {
            if existing_contact != user.contact && self.exists(&user.contact).await? {
                return Err(Error::Conflict(Self::Item::NAME));
            }
        }

        let mut primary = self.primary.write()?;
        let mut secondary = self.secondary.write()?;
        primary.insert(user.id.clone(), user.clone());
        if let Contact::Both(phone, email) = &user.contact {
            let phone = Contact::Phone(phone.clone());
            let email = Contact::Email(email.clone());
            secondary.insert(phone, user.id.clone());
            secondary.insert(email, user.id.clone());
        }else {
            secondary.insert(user.contact.clone(), user.id.clone());
        }

        Ok(user.id)
    }

    /// Reads a user by ID.
    ///
    /// # Arguments
    ///
    /// * `id` - A reference to the ID of the user to be read.
    ///
    /// # Returns
    ///
    /// * `Result<Option<User>>` - Returns the user item if found, otherwise `None`, wrapped in a `Result`.
    async fn get(
        &self,
        key: Key<&<User as Item>::PK, &<User as Item>::SK>,
    ) -> Result<Option<User>, Self::Error> {
        let pk = match key {
            Key::Pk(pk) => *pk,
            Key::Sk(sk) => {
                let secondary = self.secondary.read()?;
                match secondary.get(sk) {
                    Some(pk) => *pk,
                    None => return Ok(None),
                }
            },
            Key::Both((pk, _)) => *pk,
        };
        Ok(self.primary.read()?.get(&pk).cloned())
    }

    /// This function does nothing and whill always return None.
    /// NOT TO BE USED. IMPLEMENTED JUST FOR FORMALITY.
    async fn get_many(
        &self,
        _: Either<&<Self::Item as Item>::PK, &<Self::Item as Item>::SK>,
    ) -> Result<Option<Vec<Self::Item>>, Self::Error> {
        Ok(None)
    }

    /// Patches an existing user with the provided map of changes.
    ///
    /// # Arguments
    ///
    /// * `id` - A reference to the ID of the user to be patched.
    /// * `map` - A map containing the fields to be updated and their new values.
    ///
    /// # Returns
    ///
    /// * `Result<User>` - Returns the updated user item wrapped in a `Result`.
    async fn patch(
        &self,
        id: &<User as Item>::PK,
        mut map: Self::Map,
    ) -> Result<User, Self::Error> {
        let key = Key::Pk(id);
        if let Some(user) = self.get(key).await? {
            let id = *id;
            let username = match map.remove("username") {
                Some(name) => name.try_into()?,
                None => user.username,
            };
            let first_name = match map.remove("first_name") {
                Some(first_name) => first_name.try_into()?,
                None => user.first_name,
            };
            let last_name = match map.remove("last_name") {
                Some(name) => name.try_into()?,
                None => user.last_name,
            };
            let password = match map.remove("password") {
                Some(name) => name.try_into()?,
                None => user.password,
            };
            let contact = if map.contains_key("email") || map.contains_key("phone") || map.contains_key("phone_verified") || map.contains_key("email_verified"){
                map.try_into()?
            } else {
                user.contact
            };
            let item = User {
                id,
                username,
                first_name,
                last_name,
                contact,
                password,
            };
            self.update(&item).await?;
            return Ok(item);
        }
        Err(Error::NotFound(Self::Item::NAME))
    }

    /// Updates an existing user.
    ///
    /// # Arguments
    ///
    /// * `user` - A reference to the user item to be updated.
    ///
    /// # Returns
    ///
    /// * `Result<<User as Item>::PK>` - Returns the ID of the updated user wrapped in a `Result`.
    async fn update(&self, item: &Self::Item) -> Result<(), Self::Error> {
        let mut primary = self.primary.write()?;
        let mut secondary = self.secondary.write()?;

        if let Some(id) = secondary.get(&item.contact) {
            if id != &item.id {
                return Err(Error::Conflict(Self::Item::NAME));
            }
        }

        if let Some(existing_user) = primary.get_mut(&item.id) {
            secondary.remove(&existing_user.contact);
            *existing_user = item.clone();
            secondary.insert(item.contact.clone(), item.id.clone());
        }else {
            self.create(item).await?;
        }

        Ok(())
    }

    /// Deletes a user by ID.
    ///
    /// # Arguments
    ///
    /// * `id` - A reference to the ID of the user to be deleted.
    ///
    /// # Returns
    ///
    /// * `Result<<User as Item>::PK>` - Returns the ID of the deleted user wrapped in a `Result`.
    async fn delete(&self, id: &<User as Item>::PK) -> Result<(), Self::Error> {
        let mut primary = self.primary.write()?;
        let mut secondary = self.secondary.write()?;

        let item = match primary.remove(id){
            Some(item) => item,
            None => return  Ok(())
        };
        match &item.contact {
            Contact::Both(phone, email) => {(secondary.remove(&Contact::Phone(phone.clone())), secondary.remove(&Contact::Email(email.clone())));},
            _ => {secondary.remove(&item.contact);},
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::types::{Key, EmailAddress, User, Value, Id, Contact};
    use crate::ports::outputs::database::Table;
    use std::collections::HashMap;
    use super::Users;
    use tokio;

    #[tokio::test]
    async fn test_exists_user_contact() {
        let users = Users::new().await.unwrap();
        let user = User {
            id: Id::default(),
            username: "testuser".to_string(),
            first_name: "Test".to_string(),
            last_name: "User".to_string(),
            contact: Contact::Email(EmailAddress::new("test@example.com").unwrap()),
            password: "password".to_string(),
        };

        // Initially, the contact should not exist
        assert_eq!(users.exists(&user.contact).await.unwrap(), false);

        // Create the user
        users.create(&user).await.unwrap();

        // Now, the contact should exist
        assert_eq!(users.exists(&user.contact).await.unwrap(), true);

        // Test with a different contact
        assert_eq!(
            users
                .exists(&Contact::Email(EmailAddress::new("nonexistent@example.com").unwrap()))
                .await
                .unwrap(),
            false
        );
    }

    #[tokio::test]
    async fn test_contact_retrieval() {
        let users = Users::new().await.unwrap();
        let user = User {
            id: Id::default(),
            username: "testuser".to_string(),
            first_name: "Test".to_string(),
            last_name: "User".to_string(),
            contact: Contact::Email(EmailAddress::new("test@example.com").unwrap()),
            password: "password".to_string(),
        };

        // Initially, retrieving contact by ID should return None
        assert_eq!(users.contact(&user.id).await.unwrap(), None);

        // Create the user
        users.create(&user).await.unwrap();

        // Now, retrieving contact by ID should return the correct contact
        assert_eq!(
            users.contact(&user.id).await.unwrap(),
            Some(user.contact.clone())
        );

        // Test with a different ID
        let new_id = Id::default();
        assert_eq!(users.contact(&new_id).await.unwrap(), None);
    }

    #[tokio::test]
    async fn test_create_user() {
        let users = Users::new().await.unwrap();
        let user = User {
            id: Id::default(),
            username: "testuser".to_string(),
            first_name: "Test".to_string(),
            last_name: "User".to_string(),
            contact: Contact::Email(EmailAddress::new("test@example.com").unwrap()),
            password: "password".to_string(),
        };

        let result = users.create(&user).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_read_user() {
        let users = Users::new().await.unwrap();
        let user = User {
            id: Id::default(),
            username: "testuser".to_string(),
            first_name: "Test".to_string(),
            last_name: "User".to_string(),
            contact: Contact::Email(EmailAddress::new("test@example.com").unwrap()),
            password: "password".to_string(),
        };

        let id = users.create(&user).await.unwrap();
        let key = Key::Pk(&id);
        let read_user = users.get(key).await.unwrap();
        assert_eq!(Some(user), read_user);
    }

    #[tokio::test]
    async fn test_patch_user() {
        let users = Users::new().await.unwrap();
        let user = User {
            id: Id::default(),
            username: "testuser".to_string(),
            first_name: "Test".to_string(),
            last_name: "User".to_string(),
            contact: Contact::Email(EmailAddress::new("test@example.com").unwrap()),
            password: "password".to_string(),
        };

        // Create the user
        users.create(&user).await.unwrap();

        // Prepare a map with changes
        let mut changes = HashMap::new();
        changes.insert(
            "username".to_string(),
            Value::String("updateduser".to_string()),
        );
        changes.insert(
            "email".to_string(),
            Value::String("updated@example.com".to_string()),
        );

        // Patch the user
        let patched_user = users.patch(&user.id, changes).await.unwrap();

        // Verify the changes
        assert_eq!(patched_user.username, "updateduser");
        assert_eq!(
            patched_user.contact,
            Contact::Email(EmailAddress::new("updated@example.com").unwrap())
        );
    }

    #[tokio::test]
    async fn test_update_user() {
        let users = Users::new().await.unwrap();
        let mut user = User {
            id: Id::default(),
            username: "testuser".to_string(),
            first_name: "Test".to_string(),
            last_name: "User".to_string(),
            contact: Contact::Email(EmailAddress::new("test@example.com").unwrap()),
            password: "password".to_string(),
        };

        let id = users.create(&user).await.unwrap();
        let key = Key::Pk(&id);
        user.contact = Contact::Email(EmailAddress::new("newcontact@example.com").unwrap());
        let update_result = users.update(&user).await;
        assert!(update_result.is_ok());

        let updated_user = users.get(key).await.unwrap();
        assert_eq!(Some(user), updated_user);
    }

    #[tokio::test]
    async fn test_delete_user() {
        let users = Users::new().await.unwrap();
        let user = User {
            id: Id::default(),
            username: "testuser".to_string(),
            first_name: "Test".to_string(),
            last_name: "User".to_string(),
            contact: Contact::Email(EmailAddress::new("test@example.com").unwrap()),
            password: "password".to_string(),
        };

        let id = users.create(&user).await.unwrap();
        let key = Key::Pk(&id);
        let delete_result = users.delete(&id).await;
        assert!(delete_result.is_ok());

        let deleted_user = users.get(key).await.unwrap();
        assert!(deleted_user.is_none());
    }
}
