use lettre::{message::{Mailbox, Message, SinglePart}, transport::smtp::{authentication::Credentials, PoolConfig}, Address, AsyncSmtpTransport, AsyncTransport, Tokio1Executor};
use crate::domain::types::EmailAddress;
use serde::Serialize;
use super::Error;
use std::fmt;

mod deserialize;
mod verify;

type Client = AsyncSmtpTransport<Tokio1Executor>;

#[derive(Clone, Debug, Serialize)]
pub struct Smtp {
    url: String,
    credentials: Option<Credentials>,
    sender: Mailbox,
    #[serde(skip)]
    client: Client
}

impl Smtp {
    /// Creates a new SMTP client from the given configuration
    pub fn new(url: String, credentials: Option<Credentials>, sender: Mailbox) -> Result<Self, Error> {
        let mut mailer = Client::from_url(&url)?;
        if let Some(credentials) = credentials.clone() {
            mailer = mailer.credentials(credentials);
        }
        let client = mailer.pool_config(PoolConfig::new()).build();
        
        Ok(Self {
            url,
            credentials,
            sender,
            client
        })
    }
    
    /// Sends an email with the given details
    pub async fn send_email(&self, to: Mailbox, subject: String, body: String) -> Result<(), Error> {
        let part = SinglePart::html(body);
        let email = Message::builder()
            .from(self.sender.clone())
            .to(to)
            .subject(subject)
            .singlepart(part)?;
            
        self.client.send(email).await?;
        Ok(())
    }
    
    /// Creates a verification email with the given code
    pub fn create_verification_email(&self, code: &str) -> String {
        format!(
            "<html><body><h1>Verification Code</h1><p>Your verification code is: <strong>{}</strong></p></body></html>",
            code
        )
    }
}

impl Default for Smtp {
    fn default() -> Self {
        let url = String::from("smtp://localhost:25");
        let credentials = None;
        let email = Address::new("noreply", "example.com").expect("Failed to create default email address");
        let sender = Mailbox::new(Some(String::from("Verification")), email);
        let client = Client::from_url(&url)
            .expect("Failed to create default SMTP client")
            .pool_config(PoolConfig::new())
            .build();
            
        Self {
            url,
            credentials,
            sender,
            client
        }
    }
}
