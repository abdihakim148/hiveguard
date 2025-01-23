use serde::{de::DeserializeOwned, Deserialize, Serialize};
use crate::ports::inputs::config::Config as ConfigTrait;
use crate::ports::outputs::database::Database;
use super::{argon::Argon, Paseto};
use std::io::{Read, Write};



#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Config<DB: Database + Default> {
    database: DB,
    argon: Argon,
    paseto: Paseto
}



impl<DB: Database + Default> Config<DB> {
    pub fn db(&self) -> &DB {
        &self.database
    }

    pub fn argon(&self) -> &Argon {
        &self.argon
    }

    pub fn paseto(&self) -> &Paseto {
        &self.paseto
    }
}


impl<DB: Database + Default + Serialize + DeserializeOwned> ConfigTrait for Config<DB> {
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
impl<DB: Database + Default + Serialize + DeserializeOwned> Default for Config<DB> {
    fn default() -> Self {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        let config = runtime.block_on(Config::load(None, ())).unwrap();
        config
    }
}