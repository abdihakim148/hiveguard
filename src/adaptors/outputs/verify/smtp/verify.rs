use crate::domain::types::{EmailAddress, Verification, Either, Key};
use crate::ports::outputs::database::{CreateItem, GetItem};
use crate::ports::{outputs::verify::{Verify, Code}};
use lettre::message::Mailbox;
use super::{Smtp, Error};


const SUBJECT: &'static str = "Verification";



impl Verify<EmailAddress> for Smtp {
    type Error = Error;
    type Channel = ();  // No channel selection needed for email
    type Verification = Verification<String>;

    async fn initiate<DB: CreateItem<Self::Verification>>(
        &self,
        email: &EmailAddress,
        _channel: Self::Channel,
        db: &DB
    ) -> Result<(), Self::Error> {
        // Create a new verification code
        let mut verification = Self::Verification::new(email, None, String::new());
        let code = Code::<EmailAddress>::as_str(&verification);
        
        // Create email content
        let to = Mailbox::new(None, email.clone().into());
        let subject = String::from(SUBJECT);
        let body = self.create_verification_email(&code);
        
        // Send the email
        self.send_email(to, subject, body).await?;
        
        // Store the verification in the database
        db.create_item(verification).await.map_err(Error::err)?;
        
        Ok(())
    }

    async fn verify<DB: GetItem<Self::Verification>>(
        &self,
        email: &EmailAddress,
        code: &str,
        db: &DB
    ) -> Result<(), Self::Error> {
        // Retrieve the verification from the database
        let contact = Either::Right(email.clone());
        let key = Key::Pk(&contact);
        let verification = db.get_item(key).await.map_err(Self::Error::err)?;
        
        // Compare the codes
        let saved_code = Code::<EmailAddress>::as_str(&verification);
        if saved_code != code {
            return Err(Error::InvalidCode);
        }
        
        // If we get here, the code is valid
        Ok(())
    }
}
