use serde::{de::{self, DeserializeOwned, Visitor}, ser::SerializeStruct, Deserialize, Serialize};
use crate::ports::inputs::config::Config as ConfigTrait;
use super::{argon::Argon, Paseto, mail::MailConfig};
use crate::ports::outputs::mailer::Mailer;
use crate::domain::types::Mail;
use std::io::{Read, Write};




pub struct Config<DB, M> {
    pub name: String,
    domain: String,
    database: DB,
    argon: Argon,
    paseto: Paseto,
    mailer: MailConfig<M>
}


impl<DB, M> Config<DB, M> {
    pub fn db(&self) -> &DB {
        &self.database
    }

    pub fn argon(&self) -> &Argon {
        &self.argon
    }

    pub fn paseto(&self) -> &Paseto {
        &self.paseto
    }

    pub fn mailer(&self) -> &M {
        &self.mailer.mailer()
    }
}



impl<DB, M> Config<DB, M>
where 
    DB: Default + Serialize + DeserializeOwned,
    M: Mailer + TryFrom<Mail>,
    <M as TryFrom<Mail>>::Error: std::fmt::Display + std::fmt::Debug
{


    fn load_sync(path: Option<&str>, input: <Self as ConfigTrait>::Input) -> Result<Self, <Self as ConfigTrait>::Error> {
        let path = match path {Some(path) => path, None => <Self as ConfigTrait>::PATH};
        match std::fs::File::open(path) {
            Ok(mut file) => {
                let mut buf = String::new();
                file.read_to_string(&mut buf)?;
                Ok(serde_json::from_str::<Self>(&buf)?)
            },
            _ => {
                let config = <Self as Default>::default();
                let path = Some(path);
                config.save_sync(path, input)?;
                Ok(config)
            }
        }
    }

    fn save_sync(&self, path: Option<&str>, _input: <Self as ConfigTrait>::Input) -> Result<(), <Self as ConfigTrait>::Error> {
        let path = match path {Some(path) => path, None => Self::PATH};
        let json = serde_json::to_string(&self)?;
        let mut file = std::fs::OpenOptions::new().write(true).create(true).open(path)?;
        let buf = json.as_bytes();
        file.write_all(buf)?;
        Ok(())
    }
}


impl<DB: Serialize, M: Mailer + TryFrom<Mail>> Serialize for Config<DB, M> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer {
        let mut state = serializer.serialize_struct("Config", 4)?;
        state.serialize_field("database", &self.database)?;
        state.serialize_field("argon", &self.argon)?;
        state.serialize_field("paseto", &self.paseto)?;
        state.serialize_field("mailer", &self.mailer)?;
        state.end()
    }
}


impl<DB: Default + Serialize + DeserializeOwned, M> ConfigTrait for Config<DB, M> 
where 
    DB: Default + Serialize + DeserializeOwned,
    M: Mailer + TryFrom<Mail>,
    <M as TryFrom<Mail>>::Error: std::fmt::Display + std::fmt::Debug
{
    type Error = Box<dyn std::error::Error + 'static>;
    type Input = ();
    

    async  fn load(path: Option<&str>, input: Self::Input) -> Result<Self, Self::Error> {
        Self::load_sync(path, input)
    }

    async fn save(&self, path: Option<&str>, input: Self::Input) -> Result<(), Self::Error> {
        self.save_sync(path, input)
    }
}


impl<DB: Default , M: Mailer + TryFrom<Mail>> Default for Config<DB, M> 
where 
    <M as TryFrom<Mail>>::Error: std::fmt::Display + std::fmt::Debug
{
    fn default() -> Self {
        let name = String::from("Beekeeper");
        let domain = Default::default();
        let database = Default::default();
        let argon = Default::default();
        let paseto = Default::default();
        let mailer = Default::default();

        Self{name, domain, database, argon, paseto, mailer}
    }
}


impl<'de, DB, M> Deserialize<'de> for Config<DB, M> 
where
    DB: Default + Deserialize<'de>,
    M: Mailer + TryFrom<Mail>,
    <M as TryFrom<Mail>>::Error: std::fmt::Display + std::fmt::Debug
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de> {

        struct ConfigVisitor<DB, M> {
            _t: std::marker::PhantomData<(DB, M)>
        }

        impl<'de, DB, M> Visitor<'de> for ConfigVisitor<DB, M> 
        where
            DB: Default + Deserialize<'de>,
            M: Mailer + TryFrom<Mail>,
            <M as TryFrom<Mail>>::Error: std::fmt::Display + std::fmt::Debug
        {
            type Value = Config<DB, M>;
            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a struct of Config")
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
                where
                    A: serde::de::MapAccess<'de>, {
                let mut name = None;
                let mut domain = None;
                let mut database = None;
                let mut argon = None;
                let mut paseto = None;
                let mut mailer = None;

                while let Some(key) = map.next_key()? {
                    match key {
                        "name" => {
                            if name.is_some() {
                                return Err(de::Error::duplicate_field("name"));
                            }
                            name = map.next_value()?;
                        },
                        "domain" => {
                            if domain.is_some() {
                                return Err(de::Error::duplicate_field("domain"));
                            }
                            domain = map.next_value()?;
                        },
                        "database" => {
                            if database.is_some() {
                                return Err(de::Error::duplicate_field("database"));
                            }
                            database = map.next_value()?;
                        },
                        "argon" => {
                            if argon.is_some() {
                                return Err(de::Error::duplicate_field("argon"));
                            }
                            argon = map.next_value()?;
                        },
                        "paseto" => {
                            if paseto.is_some() {
                                return Err(de::Error::duplicate_field("paseto"));
                            }
                            paseto = map.next_value()?;
                        },
                        "mailer" => {
                            if mailer.is_some() {
                                return Err(de::Error::duplicate_field("mailer"));
                            }
                            mailer = map.next_value()?;
                        },
                        _ => {
                            let _: de::IgnoredAny = map.next_value()?;
                        }
                    }
                }

                let name = name.unwrap_or_default();
                let domain = domain.unwrap_or_default();
                let database = database.unwrap_or_default();
                let argon = argon.unwrap_or_default();
                let paseto = paseto.unwrap_or_default();
                let mailer = mailer.unwrap_or_default();

                Ok(Config{name, domain, database, argon, paseto, mailer})
            }
        }
        let visitor = ConfigVisitor::<DB, M>{_t: std::marker::PhantomData::default()};
        deserializer.deserialize_map(visitor)
    }
}