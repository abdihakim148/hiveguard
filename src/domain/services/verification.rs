use crate::ports::{ErrorTrait, outputs::{verify::{Verify, Code}, database::{CreateItem, GetItem, UpdateItem, Item}}};
use super::super::types::{User, Either, EmailAddress, Phone, Error, Value, Key, Contact};
use std::collections::HashMap;

pub trait Verification<T: Clone, E: Item, const DIGITS: usize = 6>: Verify<T, DIGITS> {
    type Error: ErrorTrait;
    async fn initiate_verification<DB: CreateItem<Self::Verification> + GetItem<E>>(&self, contact: T, channel: <Self as Verify<T, DIGITS>>::Channel, base_url: &str, db: &DB) -> Result<(), <Self as Verification<T, E, DIGITS>>::Error>;
    async fn confirm_verification<DB: GetItem<Self::Verification> + UpdateItem<E, Update = HashMap<String, Value>>>(&self, contact: T, code: Either<&str, &<Self::Verification as Code<T, DIGITS>>::Id>, db: &DB) -> Result<E, <Self as Verification<T, E, DIGITS>>::Error>;
}

#[cfg(feature = "email")]
impl<V: Verify<EmailAddress>> Verification<EmailAddress, User> for V {
    type Error = Error;

    async fn initiate_verification<DB: CreateItem<Self::Verification> + GetItem<User>>(&self, contact: EmailAddress, channel: <Self as Verify<EmailAddress, 6>>::Channel, base_url: &str, db: &DB) -> Result<(), <Self as Verification<EmailAddress, User, 6>>::Error> {
        // make sure a user with this contact exists before trying to create one.
        let contact = Contact::Email(contact);
        let email = match &contact {
            Contact::Email(email) => email,
            _ => panic!("THIS SHOULD NEVER HAPPEN")
        };
        let key = Key::Sk(&contact);
        // just making sure that a user with the provided contact exists.
        let user = db.get_item(key).await.map_err(Error::new)?.ok_or(Error::item_not_found(User::NAME))?;
        if user.contact.verified()? {
            return Err(Error::ContactAlreadyVerified)
        }
        <Self as Verify<EmailAddress>>::initiate(&self, email, channel, base_url, db).await.map_err(Error::new)
    }

    async fn confirm_verification<DB: GetItem<Self::Verification> + UpdateItem<User, Update = HashMap<String, Value>>>(&self, contact: EmailAddress, code: Either<&str, &<Self::Verification as Code<EmailAddress, 6>>::Id>, db: &DB) -> Result<User, <Self as Verification<EmailAddress, User, 6>>::Error> {
        <Self as Verify<EmailAddress>>::verify(&self, &contact, code, db).await.map_err(Error::new)?;
        let contact = Contact::Email(contact);
        let address: &str = match &contact {
            Contact::Email(email) => email.as_ref(),
            _ => panic!("THIS SHOULD NEVER HAPPEN"),
        };
        let key = Key::Sk(&contact);
        let update = [(String::from("email"), Value::String(String::from(address))), (String::from("email_verified"), Value::Bool(true))].into();
        let mut user = db.patch_item(key, update).await.map_err(Error::new)?;
        user.password = Default::default();
        Ok(user)
    }
}



#[cfg(feature = "phone")]
impl<V: Verify<Phone>> Verification<Phone, User> for V {
    type Error = Error;

    async fn initiate_verification<DB: CreateItem<Self::Verification> + GetItem<User>>(&self, contact: Phone, channel: <Self as Verify<Phone, 6>>::Channel, base_url: &str, db: &DB) -> Result<(), <Self as Verification<Phone, User, 6>>::Error> {
        // make sure a user with this contact exists before trying to create one.
        let contact = Contact::Phone(contact);
        let phone = match &contact {
            Contact::Phone(phone) => phone,
            _ => panic!("THIS SHOULD NEVER HAPPEN")
        };
        let key = Key::Sk(&contact);
        // just making sure that a user with the provided contact exists.
        let _ = db.get_item(key).await.map_err(Error::new)?;
        <Self as Verify<Phone>>::initiate(&self, phone, channel, base_url, db).await.map_err(Error::new)
    }

    async fn confirm_verification<DB: GetItem<Self::Verification> + UpdateItem<User, Update = HashMap<String, Value>>>(&self, contact: Phone, _: Either<<User as Item>::PK, <User as Item>::SK>, code: Either<&str, &<Self::Verification as Code<Phone, 6>>::Id>, db: &DB) -> Result<User, <Self as Verification<Phone, User, 6>>::Error> {
        <Self as Verify<Phone>>::verify(&self, &contact, code, db).await.map_err(Error::new)?;
        let contact = Contact::Phone(contact);
        let address: &str = match &contact {
            Contact::Phone(phone) => phone.as_ref(),
            _ => panic!("THIS SHOULD NEVER HAPPEN"),
        };
        let key = Key::Sk(&contact);
        let update = [(String::from("phone"), Value::String(String::from(address))), (String::from("phone_verified"), Value::Bool(true))].into();
        let user = db.patch_item(key, update).await.map_err(Error::new)?;
        Ok(user)
    }
}