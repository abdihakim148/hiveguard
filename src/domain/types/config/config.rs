use serde::{de::DeserializeOwned, Deserialize, Serialize};
use crate::ports::inputs::config::Config as ConfigTrait;
use super::{argon::Argon, Paseto, mail::MailConfig};
use crate::ports::outputs::database::Database;
use crate::ports::outputs::mailer::Mailer;
use crate::domain::types::Mail;
use std::io::{Read, Write};



#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Config<DB: Database + Default, M: Mailer + TryFrom<Mail>> 
where M::Error: std::fmt::Display
{
    database: DB,
    argon: Argon,
    paseto: Paseto,
    mailer: MailConfig<M>
}



impl<DB: Database + Default, M: Mailer + TryFrom<Mail>> Config<DB, M> 
where M::Error: std::fmt::Display
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
}


impl<DB: Database + Default + Serialize + DeserializeOwned, M> ConfigTrait for Config<DB, M> 
where 
    M: Mailer + TryFrom<Mail> + Serialize + DeserializeOwned,
    M::Error: std::fmt::Display,
{
    type Error = Box<dyn std::error::Error + 'static>;
    type Input = ();
    

    async  fn load(path: Option<&str>, input: Self::Input) -> Result<Self, Self::Error> {
        let path = match path {Some(path) => path, None => Self::PATH};
        match std::fs::File::open(path) {
            Ok(mut file) => {
                let mut buf = String::new();
                file.read_to_string(&mut buf)?;
                Ok(serde_json::from_str::<Self>(&buf)?)
            },
            _ => {
                let config = <Self as Default>::default();
                let path = Some(path);
                config.save(path, input).await?;
                Ok(config)
            }
        }
    }

    async fn save(&self, path: Option<&str>, _input: Self::Input) -> Result<(), Self::Error> {
        let path = match path {Some(path) => path, None => Self::PATH};
        let json = serde_json::to_string(&self)?;
        let mut file = std::fs::OpenOptions::new().write(true).create(true).open(path)?;
        let buf = json.as_bytes();
        file.write_all(buf)?;
        Ok(())
    }
}


/// This implementations panics.
impl<DB: Database + Default + Serialize + DeserializeOwned, M> Default for Config<DB, M> 
where 
    M: Mailer + TryFrom<Mail> + Serialize + DeserializeOwned,
    M::Error: std::fmt::Display,
{
    fn default() -> Self {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        let config = runtime.block_on(Config::load(None, ())).unwrap();
        config
    }
}