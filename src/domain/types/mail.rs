use lettre::{message::Mailbox, transport::smtp::authentication::Credentials};
use serde::{Serialize, Deserialize, Deserializer, de::{self, MapAccess, Visitor}};
use std::fmt;


#[derive(Debug, Clone, Serialize)]
pub struct Mail {
    url: String,
    credentials: Option<Credentials>,
    sender: Mailbox
}


impl<'de> Deserialize<'de> for Mail {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct MailData {
            credentials: Option<Credentials>,
            url: String,
            sender: Mailbox,
        }

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
                            let res: Result<Cred, V::Error> = map.next_value();
                            credentials = match res {
                                Ok(value) => Some(value.into()),
                                _ => Some(map.next_value()?)
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

    #[test]
    fn test_mail_serialization() {
        let mail = Mail {
            url: "smtp://example.com".to_string(),
            credentials: Some(Credentials::new("user".to_string(), "password".to_string())),
            sender: "sender@example.com".parse::<Mailbox>().unwrap(),
        };

        let serialized = serde_json::to_string(&mail).unwrap();
        println!("JSON: {}", serialized);
        assert!(serialized.contains("\"url\":\"smtp://example.com\""));
        assert!(serialized.contains("\"credentials\":"));
        assert!(serialized.contains("\"sender\":\"sender@example.com\""));
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
                "name": "User",
                "email": "sender@example.com"
            }
        }
        "#;

        let mail: Mail = serde_json::from_str(data).unwrap();
        assert_eq!(mail.url, "smtp://example.com");
        assert!(mail.credentials.is_some());
        assert_eq!(mail.sender.to_string(), "User <sender@example.com>");
    }
}
