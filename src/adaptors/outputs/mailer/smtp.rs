use lettre::{message::{Mailbox, Message, SinglePart}, transport::smtp::PoolConfig, AsyncSmtpTransport, Tokio1Executor, AsyncTransport};
use crate::ports::outputs::mailer::Mailer;
use crate::domain::types::{Error, Value};
use serde::{Serialize, Deserialize};
use crate::domain::types::Mail;
use crate::ports::Result;


type Client = AsyncSmtpTransport<Tokio1Executor>;

/// This is the default SmtpMailer Client
#[derive(Debug, Clone)]
pub struct SmtpMailer(Client);


impl Mailer for SmtpMailer {
    type Config = Mail;
    /// sender, receiver, subject, body
    type Mail = (Mailbox, Mailbox, String, String);

    async fn new(mail: Self::Config) -> Result<Self> {
        Ok(mail.try_into()?)
    }

    async fn send(&self, mail: Self::Mail) -> Result<()> {
        let sender = mail.0;
        let receiver = mail.1;
        let subject = mail.2;
        let body = mail.3;
        let part = SinglePart::html(body);
        let email = Message::builder()
        .from(sender)
        .to(receiver)
        .subject(subject)
        .singlepart(part).map_err(|err|Error::from(err))?;
        self.0.send(email).await.map_err(|err|Error::from(err))?;
        Ok(())
    }
}


impl TryFrom<Mail> for SmtpMailer {
    type Error = Error;

    fn try_from(mail: Mail) -> std::result::Result<Self, Self::Error> {
        let connection_url = &mail.url;
        let mut mailer = Client::from_url(connection_url)?;
        if let Some(credentials) = mail.credentials {
            mailer = mailer.credentials(credentials)
        }
        let client = mailer.pool_config(PoolConfig::new()).build();
        Ok(SmtpMailer(client))
    }
}

/// This implementation should never be used.
/// It is implemented just to satisfy a trait requirement
impl Serialize for SmtpMailer {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
        where
            S: serde::Serializer {
        serializer.serialize_str("")
    }
}


/// This implementation should never be used.
/// It is implemented just to satisfy a trait requirement
/// This will always result in an error.
impl<'de> Deserialize<'de> for SmtpMailer {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
        where
            D: serde::Deserializer<'de> {
        Err(serde::de::Error::custom("This operation should never be used"))
    }
}