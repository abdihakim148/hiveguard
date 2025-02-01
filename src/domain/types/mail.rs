use lettre::{message::Mailbox, transport::smtp::authentication::Credentials, Address};
use serde::{Serialize, Deserialize, Deserializer, de::{self, MapAccess, Visitor}};
use std::fmt;


#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct Mail {
    pub url: String,
    pub credentials: Option<Credentials>,
    pub sender: Mailbox
}


impl Default for Mail {
    fn default() -> Self {
        let url = String::from("smtp://example.com");
        let credentials = None;
        let email = Address::new("sender", "example.com").expect("PANIC ON DEFAULT MAIL SENDER");
        let sender = Mailbox::new(Some(String::from("Sender")), email);
        Self {url, credentials, sender}
    }
}


impl<'de> Deserialize<'de> for Mail {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {

        struct MailVisitor;

        #[derive(Deserialize)]
        struct Cred {
            #[serde(alias = "user", alias = "user_name")]
            name: String,
            #[serde(alias = "secret")]
            password: String
        }

        impl From<Cred> for Credentials {
            fn from(cred: Cred) -> Self {
                (cred.name, cred.password).into()
            }
        }

        impl<'de> Visitor<'de> for MailVisitor {
            type Value = Mail;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct Mail")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<Mail, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut credentials = None;
                let mut url = Option::<String>::None;
                let mut sender = None;

                while let Some(key) = map.next_key()? {
                    match key {
                        "credentials" => {
                            let res: Result<Option<Cred>, V::Error> = map.next_value();
                            credentials = match res {
                                Ok(value) => {
                                    match value {
                                        Some(value) => Some(value.into()),
                                        None => None,
                                    }
                                },
                                _ => map.next_value()?
                            };
                        }
                        "url" => {
                            url = map.next_value()?;
                        }
                        "sender" => {
                            sender = Some(map.next_value()?);
                        }
                        _ => {
                            let _: de::IgnoredAny = map.next_value()?;
                        }
                    }
                }

                let credentials = credentials;
                let url = url.ok_or_else(|| de::Error::missing_field("url"))?;
                let sender = sender.ok_or_else(|| de::Error::missing_field("sender"))?;

                

                Ok(Mail {
                    credentials,
                    url,
                    sender,
                })
            }
        }

        const FIELDS: &'static [&'static str] = &["credentials", "url", "sender"];
        deserializer.deserialize_struct("Mail", FIELDS, MailVisitor)
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;
    use lettre::message::Mailbox;
    use lettre::transport::smtp::authentication::Credentials;

    impl Mail {

        pub fn new_default() -> Self {
            Mail {
                url: "smtp://example.com".to_string(),
                credentials: None,
                sender: Mailbox::new(Some(String::from("Sender")), Address::new("sender", "example.com").unwrap()),
            }
        }

        pub fn set_default_credentials(&mut self) {
            self.credentials = Some(Credentials::new("smtp".to_string(), "password".to_string()))
        }
    }

    #[test]
    fn test_mail_deserialization() {
        let data = r#"
        {
            "url": "smtp://example.com",
            "credentials": {
                "user_name": "smtp",
                "password": "password"
            },
            "sender": {
                "name": "Sender",
                "email": "sender@example.com"
            }
        }
        "#;

        let mail: Mail = serde_json::from_str(data).unwrap();
        let mut default_mail = Mail::new_default();
        default_mail.set_default_credentials();
        assert_eq!(mail, default_mail);
    }
}
