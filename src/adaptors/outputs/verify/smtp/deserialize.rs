use serde::{Serialize, Deserialize, Deserializer, de::{self, MapAccess, Visitor}};
use std::fmt;
use lettre::{message::Mailbox, transport::smtp::authentication::Credentials, Address};
use super::Smtp;

impl<'de> Deserialize<'de> for Smtp {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct SmtpVisitor;

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

        impl<'de> Visitor<'de> for SmtpVisitor {
            type Value = Smtp;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct Smtp")
            }

            fn visit_map<V>(self, mut map: V) -> Result<Smtp, V::Error>
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

                let url = url.ok_or_else(|| de::Error::missing_field("url"))?;
                let sender = sender.ok_or_else(|| de::Error::missing_field("sender"))?;

                Smtp::new(url, credentials, sender).map_err(de::Error::custom)
            }
        }

        const FIELDS: &'static [&'static str] = &["credentials", "url", "sender"];
        deserializer.deserialize_struct("Smtp", FIELDS, SmtpVisitor)
    }
}
