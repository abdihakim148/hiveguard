/// Module for database tables.
mod tables;
// mod error;

use crate::ports::outputs::database::Database;
use crate::ports::outputs::database::Table;
use serde::{Serialize, Deserialize};
use crate::domain::types::Error;
pub use tables::*;

/// A struct representing an in-memory database.
/// A struct representing an in-memory database.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Memory {
    #[serde(skip)]
    users: Users,
    #[serde(skip)]
    verifications: Verifications,
    #[serde(skip)]
    organisations: Organisations
}

impl Database for Memory {
    type Verifications = Verifications;
    type Organisations = Organisations;
    type Resources = Resources;
    type Services = Services;
    type Members = Members;
    type Scopes = Scopes;
    type Users = Users;
    type Roles = Roles;
    type Config = ();
    type Error = Error;

    async fn new(_config: ()) -> Result<Self, Self::Error> {
        let users = Users::new().await?;
        let verifications = Verifications::new().await?;
        let organisations = Organisations::new().await?;
        Ok(Memory{users, verifications, organisations})
    }

    async fn verifications<'a>(&'a self) -> Result<&'a Self::Verifications, Self::Error> {
        Ok(&self.verifications)
    }

    async fn organisations<'a>(&'a self) -> Result<&'a Self::Organisations, Self::Error> {
        Ok(&self.organisations)
    }

    async fn resources<'a>(&'a self) -> Result<&'a Self::Resources, Self::Error> {
        todo!()
    }

    async fn services<'a>(&'a self) -> Result<&'a Self::Services, Self::Error> {
        todo!()
    }

    async fn members<'a>(&'a self) -> Result<&'a Self::Members, Self::Error> {
        todo!()
    }

    async fn scopes<'a>(&'a self) -> Result<&'a Self::Scopes, Self::Error> {
        todo!()
    }

    async fn users<'a>(&'a self) -> Result<&'a Self::Users, Self::Error> {
        Ok(&self.users)
    }

    async fn roles<'a>(&'a self) -> Result<&'a Self::Roles, Self::Error> {
        todo!()
    }
}
