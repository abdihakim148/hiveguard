use crate::ports::outputs::database::Database;
use serde::{Serialize, Deserialize};
use super::{argon::Argon, Paseto};



#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct Config<DB: Database> {
    database: DB,
    argon: Argon,
    paseto: Paseto
}



impl<DB: Database> Config<DB> {
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