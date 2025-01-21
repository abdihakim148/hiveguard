use crate::ports::outputs::database::Database;
use serde::{Serialize, Deserialize};
use super::{argon::Argon, Paseto};



#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Config<DB: Database> {
    database: DB,
    argon: Argon,
    paseto: Paseto
}