use serde::{de::DeserializeOwned, Deserialize, Serialize};
use crate::ports::inputs::config::Config as ConfigTrait;
use super::{argon::Argon, Paseto, mail::MailConfig};
use crate::ports::outputs::database::Database;
use crate::ports::outputs::mailer::Mailer;
use crate::domain::types::Mail;
use std::io::{Read, Write};



#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Config<DB: Database + Default, M: Mailer + TryFrom<Mail>> 
where M::Error: std::fmt::Display + std::fmt::Debug,
{
    database: DB,
    argon: Argon,
    paseto: Paseto,
    mailer: MailConfig<M>
}



impl<DB: Database + Default + Serialize + DeserializeOwned, M: Mailer + TryFrom<Mail>> Config<DB, M> 
where 
    M: Mailer + TryFrom<Mail> + Serialize + DeserializeOwned,
    M::Error: std::fmt::Display + std::fmt::Debug,
{
    pub fn db(&self) -> &DB {
        &self.database
    }

    pub fn argon(&self) -> &Argon {
        &self.argon
    }

    pub fn paseto(&self) -> &Paseto {
        &self.paseto
    }

    pub fn mailer(&self) -> &MailConfig<M> {
        &self.mailer
    }

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


impl<DB: Database + Default + Serialize + DeserializeOwned, M> ConfigTrait for Config<DB, M> 
where 
    M: Mailer + TryFrom<Mail> + Serialize + DeserializeOwned,
    M::Error: std::fmt::Display + std::fmt::Debug,
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


impl<DB: Default + Database, M: Mailer + TryFrom<Mail>> Default for Config<DB, M> 
where 
    M::Error: std::fmt::Display + std::fmt::Debug
{
    fn default() -> Self {
        let database = Default::default();
        let argon = Default::default();
        let paseto = Default::default();
        let mailer = Default::default();

        Self{database, argon, paseto, mailer}
    }
}