use crate::domain::types::{EmailAddress, Verification, Either, Key, Id, VerificationMedia};
use crate::ports::outputs::database::{CreateItem, GetItem};
use crate::ports::{outputs::verify::{Verify, Code}};
use lettre::message::Mailbox;
use super::{Smtp, Error};


const SUBJECT: &'static str = "Verification";



impl Verify<EmailAddress> for Smtp {
    type Error = Error;
    type Channel = VerificationMedia;  // No channel selection needed for email
    type Verification = Verification<Id>;

    async fn initiate<DB: CreateItem<Self::Verification>>(
        &self,
        email: &EmailAddress,
        _channel: Self::Channel,
        base_url: &str,
        db: &DB
    ) -> Result<(), Self::Error> {
        // Create a new verification code
        let mut verification = Self::Verification::new(email, None, Id::default());
        let code = Code::<EmailAddress>::as_str(&verification);
        let base_url = base_url.trim_end_matches('/');
        let contact: &str = email.as_ref();
        let link = format!("{}/{}?contact={}", base_url, verification.id.to_hex(), contact);
        
        // Create email content
        let to = Mailbox::new(None, email.clone().into());
        let subject = String::from(SUBJECT);
        let body = self.create_verification_email(&code, &link);
        
        // Send the email
        self.send_email(to, subject, body).await?;
        
        // Store the verification in the database
        db.create_item(verification).await.map_err(Error::err)?;
        
        Ok(())
    }

    async fn verify<DB: GetItem<Self::Verification>>(
        &self,
        email: &EmailAddress,
        code: Either<&str, &Id>,
        db: &DB
    ) -> Result<(), Self::Error> {
        // Retrieve the verification from the database
        let contact = Either::Right(email.clone());
        let key = Key::Pk(&contact);
        let verification = db.get_item(key).await.map_err(Self::Error::err)?.ok_or(Error::InvalidCode)?;

        let valid = match code {
            Either::Left(code) => {
                let saved_code = Code::<EmailAddress>::as_str(&verification);
                saved_code.as_str() == code
            },
            Either::Right(id) => verification.id == *id,
        };

        if !valid {
            return Err(Error::InvalidCode);
        }
        // If we get here, the code is valid
        Ok(())
    }
}
