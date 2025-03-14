use serde::{de::{self, DeserializeOwned, Visitor}, ser::SerializeStruct, Deserialize, Serialize};
use crate::ports::inputs::config::Config as ConfigTrait;
use crate::ports::outputs::verify::Verifyer;
use super::{argon::Argon, Paseto};
use std::io::{Read, Write};



pub struct Config<DB, V> {
    pub name: String,
    domain: String,
    database: DB,
    argon: Argon,
    paseto: Paseto,
    verifyer: V,
}


impl<DB, V> Config<DB, V> {
    pub fn db(&self) -> &DB {
        &self.database
    }

    pub fn argon(&self) -> &Argon {
        &self.argon
    }

    pub fn paseto(&self) -> &Paseto {
        &self.paseto
    }

    pub fn verifyer(&self) -> &V {
        &self.verifyer
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
        state.serialize_field("database", &self.database)?;
        state.serialize_field("argon", &self.argon)?;
        state.serialize_field("paseto", &self.paseto)?;
        state.serialize_field("verifyer", &self.verifyer)?;
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
        let name = String::from("Beekeeper");
        let domain = Default::default();
        let database = Default::default();
        let argon = Default::default();
        let paseto = Default::default();
        let verifyer = Default::default();

        Self{name, domain, database, argon, paseto, verifyer}
    }
}


impl<'de, DB, V> Deserialize<'de> for Config<DB, V> 
where
    DB: Default + Deserialize<'de>,
    V: Verifyer + Default + Deserialize<'de>,
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
            V: Verifyer + Default + Deserialize<'de>,
        {
            type Value = Config<DB, V>;
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
                let mut verifyer = None;

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
                        "verifyer" => {
                            if verifyer.is_some() {
                                return Err(de::Error::duplicate_field("mailer"));
                            }
                            verifyer = map.next_value()?;
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
                let verifyer = verifyer.unwrap_or_default();

                Ok(Config{name, domain, database, argon, paseto, verifyer})
            }
        }
        let visitor = ConfigVisitor::<DB, V>{_t: std::marker::PhantomData::default()};
        deserializer.deserialize_map(visitor)
    }
}