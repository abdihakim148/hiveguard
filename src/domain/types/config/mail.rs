use serde::{Serialize, Deserialize, Deserializer, de::{self, Visitor, MapAccess}};
use crate::ports::outputs::mailer::Mailer;
use crate::domain::types::Mail;
use std::marker::PhantomData;
use std::fmt;


#[derive(Debug, Clone)]
pub struct MailConfig<M> {
    mail: Mail,
    mailer: M,
}


impl<M: Mailer + TryFrom<Mail>> PartialEq  for MailConfig<M> {
    fn eq(&self, other: &Self) -> bool {
        self.mail == other.mail
    }
}


impl<M: Mailer + TryFrom<Mail>> Default for MailConfig<M>
where
    M: Mailer + TryFrom<Mail>,
    M::Error: std::fmt::Display + std::fmt::Debug,
{
    fn default() -> Self {
        let mail = Mail::default();
        let mailer = mail.clone().try_into().expect("ERROR WHILE CONVERTING DEFAULT MAIL TO MAILER");
        Self{mail, mailer}
    }
}


impl<M: Mailer + TryFrom<Mail>> Serialize for MailConfig<M> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer {
        self.mail.serialize(serializer)
    }
}



impl<'de, M> Deserialize<'de> for MailConfig<M>
where
    M: Mailer + TryFrom<Mail>,
    M::Error: std::fmt::Display,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let mail = Mail::deserialize(deserializer)?;
        let clone = mail.clone();
        let mailer = clone.try_into().map_err(de::Error::custom)?;

        Ok(MailConfig {mail, mailer})
    }
}



#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;
    use crate::domain::types::Mail;
    use crate::ports::outputs::mailer::Mailer;
    use std::convert::TryFrom;
    use crate::ports::Result;

    #[derive(Debug, Clone)]
    struct MockMailer;

    impl Mailer for MockMailer {
        type Config = ();
        type Mail = ();

        async fn new(_config: Self::Config) -> Result<Self> {
            Ok(MockMailer)
        }

        async fn send(&self, _mail: Self::Mail) -> Result<()> {
            Ok(())
        }
    }

    impl TryFrom<Mail> for MockMailer {
        type Error = &'static str;

        fn try_from(_mail: Mail) -> std::result::Result<Self, Self::Error> {
            Ok(MockMailer)
        }
    }

    #[test]
    fn test_serialize_mail_config() {
        let mail = Mail {
            url: "smtp://example.com".to_string(),
            credentials: None,
            sender: "sender@example.com".parse().unwrap(),
        };
        let mail_config = MailConfig::<MockMailer> {
            mail: mail.clone(),
            mailer: MockMailer,
        };

        let serialized = serde_json::to_string(&mail_config).unwrap();
        let expected = serde_json::to_string(&mail).unwrap();
        assert_eq!(serialized, expected);
    }

    #[test]
    fn test_deserialize_mail_config() {
        let json = r#"
        {
            "url": "smtp://example.com",
            "credentials": null,
            "sender": "User <sender@example.com>"
        }
        "#;

        let mail_config: MailConfig<MockMailer> = serde_json::from_str(json).unwrap();
        assert_eq!(mail_config.mail.url, "smtp://example.com");
        assert_eq!(mail_config.mail.sender.to_string(), "User <sender@example.com>");
    }
}
