use serde::{Serialize, Deserialize, Deserializer, de};
use crate::ports::outputs::mailer::Mailer;
use crate::domain::types::Mail;


#[cfg_attr(test, derive(Debug))]
pub struct MailConfig<M> {
    mail: Mail,
    mailer: M,
}


impl<M> MailConfig<M>
where 
    M: Mailer + TryFrom<Mail>,
    <M as TryFrom<Mail>>::Error: std::fmt::Display + std::fmt::Debug
{
    pub fn mailer(&self) -> &M {
        &self.mailer
    }
}


impl<M> PartialEq  for MailConfig<M> {
    fn eq(&self, other: &Self) -> bool {
        self.mail == other.mail
    }
}


impl<M: Mailer + TryFrom<Mail>> Default for MailConfig<M>
where
    M: Mailer + TryFrom<Mail>,
    <M as TryFrom<Mail>>::Error: std::fmt::Display + std::fmt::Debug
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
    <M as TryFrom<Mail>>::Error: std::fmt::Display + std::fmt::Debug
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
    use crate::domain::types::{Mail, Error};
    use crate::ports::outputs::mailer::Mailer;
    use std::convert::TryFrom;

    #[derive(Debug, Clone)]
    pub struct MockMailer;

    impl Mailer for MockMailer {
        type Config = ();
        type Mail = ();
        type Error = Error;

        async fn new(_: Self::Config) -> std::result::Result<Self, Self::Error> {
            Ok(MockMailer)
        }

        async fn send(&self, _: Self::Mail) -> std::result::Result<(), Self::Error> {
            Ok(())
        }
    }

    impl TryFrom<Mail> for MockMailer {
        type Error = &'static str;

        fn try_from(_: Mail) -> std::result::Result<Self, Self::Error> {
            Ok(MockMailer)
        }
    }


    impl MailConfig<MockMailer> {
        fn new() -> Self {
            let mail = Mail::new_default();
            let mailer = MockMailer;
            MailConfig{mail, mailer}
        }
    }

    #[test]
    fn test_serialize_mail_config() {
        let mail = Mail::new_default();
        let mail_config = MailConfig::<MockMailer>::default();

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
            "sender": "Sender <sender@example.com>"
        }
        "#;

        let mail_config: MailConfig<MockMailer> = serde_json::from_str(json).unwrap();
        assert_eq!(mail_config, MailConfig::<MockMailer>::new());
    }
}
