use serde::{ser, de::{self, DeserializeOwned, Visitor}, ser::SerializeStruct, Deserialize, Serialize};
use super::{argon::Argon, Paseto as DepPaseto, Provider, Tokenizer, paseto::Paseto, jwt::Jwt};
use crate::{domain::types::config::provider, ports::inputs::config::Config as ConfigTrait};
use crate::ports::outputs::verify::Verifyer;
use std::io::{Read, Write};
use url::Host;



pub struct Config<DB, V> {
    pub name: String,
    pub host: String,
    database: DB,
    argon: Argon,
    paseto: DepPaseto,
    verifyer: V,
    oauth: Provider,
    tokenizer: Tokenizer
}


impl<DB, V> Config<DB, V> {
    pub fn db(&self) -> &DB {
        &self.database
    }

    pub fn argon(&self) -> &Argon {
        &self.argon
    }

    pub fn paseto(&self) -> &DepPaseto {
        &self.paseto
    }

    pub fn verifyer(&self) -> &V {
        &self.verifyer
    }

    pub fn oauth(&self) -> &Provider {
        &self.oauth
    }

    pub fn tokenizer(&self) -> &Tokenizer {
        &self.tokenizer
    }
}



impl<DB, V> Config<DB, V>
where 
    DB: Default + Serialize + DeserializeOwned,
    V: Verifyer + Default + Serialize + DeserializeOwned,
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


impl<DB: Serialize, V: Verifyer + Serialize> Serialize for Config<DB, V> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer {
        let mut state = serializer.serialize_struct("Config", 4)?;
        state.serialize_field("name", &self.name)?;
        if let Some((host, port)) = self.host.split_once(':') {
            let host = Host::parse(host).map_err(ser::Error::custom)?;
            let port: u16 = port.parse().map_err(ser::Error::custom)?;
            state.serialize_field("host", &host)?;
            state.serialize_field("port", &port)?;
        }
        state.serialize_field("database", &self.database)?;
        state.serialize_field("argon", &self.argon)?;
        state.serialize_field("dep_paseto", &self.paseto)?;
        state.serialize_field("verifyer", &self.verifyer)?;
        state.serialize_field("oauth", &self.oauth)?;
        match &self.tokenizer {
            Tokenizer::Jwt(jwt) => state.serialize_field("jwt", jwt)?,
            Tokenizer::Paseto(paseto) => state.serialize_field("paseto", paseto)?
        }
        state.end()
    }
}


impl<DB: Default + Serialize + DeserializeOwned, V> ConfigTrait for Config<DB, V> 
where 
    DB: Default + Serialize + DeserializeOwned,
    V: Verifyer + Default + Serialize + DeserializeOwned,
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


impl<DB: Default , V: Verifyer + Default> Default for Config<DB, V> {
    fn default() -> Self {
        let name = String::from("Hiveguard");
        let host = String::from("localhost:8080");
        let database = Default::default();
        let argon = Default::default();
        let paseto = Default::default();
        let verifyer = Default::default();
        let oauth = Default::default();
        let tokenizer = Default::default();

        Self{name, host, database, argon, paseto, verifyer, oauth, tokenizer}
    }
}


impl<'de, DB, V> Deserialize<'de> for Config<DB, V> 
where
    DB: Default + Deserialize<'de>,
    V: Verifyer + Default + DeserializeOwned,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de> {

        struct ConfigVisitor<DB, V> {
            _t: std::marker::PhantomData<(DB, V)>
        }

        impl<'de, DB, V> Visitor<'de> for ConfigVisitor<DB, V> 
        where
            DB: Default + Deserialize<'de>,
            V: Verifyer + Default + DeserializeOwned,
        {
            type Value = Config<DB, V>;
            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a struct of Config")
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
                where
                    A: serde::de::MapAccess<'de>, {
                let mut name = None;
                let mut host = None;
                let mut port = Option::<u16>::None;
                let mut database = None;
                let mut argon = None;
                let mut paseto = None;
                let mut verifyer = None;
                let mut oauth = None;
                let mut tokenizer = None;

                while let Some(key) = map.next_key()? {
                    match key {
                        "name" => {
                            if name.is_some() {
                                return Err(de::Error::duplicate_field("name"));
                            }
                            name = map.next_value()?;
                        },
                        "host" => {
                            if host.is_some() {
                                return Err(de::Error::duplicate_field("host"));
                            }
                            host = map.next_value()?;
                        },
                        "port" => {
                            if port.is_some() {
                                return Err(de::Error::duplicate_field("port"));
                            }
                            port = map.next_value()?;
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
                        "dep_paseto" => {
                            if paseto.is_some() {
                                return Err(de::Error::duplicate_field("dep_paseto"));
                            }
                            paseto = map.next_value()?;
                        },
                        "verifyer" => {
                            if verifyer.is_some() {
                                return Err(de::Error::duplicate_field("verifyer"));
                            }
                            verifyer = map.next_value()?;
                        },
                        "oauth" => {
                            if oauth.is_some() {
                                return Err(de::Error::duplicate_field("oauth"));
                            }
                            oauth = map.next_value()?;
                        },
                        "jwt" => {
                            if let Some(tokenizer) = &mut tokenizer {
                                match tokenizer {
                                    Tokenizer::Jwt(_) => return Err(de::Error::duplicate_field("jwt")),
                                    Tokenizer::Paseto(_) => return Err(de::Error::custom("cannot use both fields of `jwt` and `paseto` at the same time. you have to choose one"))
                                }
                            }
                            tokenizer = Some(Tokenizer::Jwt(map.next_value()?));
                        },
                        "paseto" => {
                            if let Some(tokenizer) = &mut tokenizer {
                                match tokenizer {
                                    Tokenizer::Jwt(_) => return Err(de::Error::custom("cannot use both fields of `paseto` and `jwt` at the same time. you have to choose one")),
                                    Tokenizer::Paseto(_) => return Err(de::Error::duplicate_field("paseto")),
                                }
                            }
                            tokenizer = Some(Tokenizer::Paseto(map.next_value()?));
                        }
                        _ => {
                            let _: de::IgnoredAny = map.next_value()?;
                        }
                    }
                }

                let name = name.unwrap_or_default();
                let host = host.unwrap_or(Host::parse("localhost").expect("THIS SHOULD NEVER PANIC. `localhost` is a valid host name"));
                let database = database.unwrap_or_default();
                let argon = argon.unwrap_or_default();
                let paseto = paseto.unwrap_or_default();
                let verifyer = verifyer.unwrap_or_default();
                let oauth = oauth.unwrap_or_default();
                let tokenizer = tokenizer.unwrap_or_default();
                let host = match port {
                    Some(port) => format!("{}:{}", host, port),
                    None => format!("{}", host)
                };



                Ok(Config{name, host, database, argon, paseto, verifyer, oauth, tokenizer})
            }
        }
        let visitor = ConfigVisitor::<DB, V>{_t: std::marker::PhantomData::default()};
        deserializer.deserialize_map(visitor)
    }
}
