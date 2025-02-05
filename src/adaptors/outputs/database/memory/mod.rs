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
    verifications: Verifications,
    #[serde(skip)]
    organisations: Organisations,
    #[serde(skip)]
    resources: Resources,
    #[serde(skip)]
    services: Services,
    #[serde(skip)]
    members: Members,
    #[serde(skip)]
    users: Users
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
        let verifications = Verifications::new().await?;
        let organisations = Organisations::new().await?;
        let resources = Resources::new().await?;
        let services = Services::new().await?;
        let members = Members::new().await?;
        let users = Users::new().await?;
        Ok(Memory{verifications, organisations, resources, services, members, users})
    }

    async fn verifications<'a>(&'a self) -> Result<&'a Self::Verifications, Self::Error> {
        Ok(&self.verifications)
    }

    async fn organisations<'a>(&'a self) -> Result<&'a Self::Organisations, Self::Error> {
        Ok(&self.organisations)
    }

    async fn resources<'a>(&'a self) -> Result<&'a Self::Resources, Self::Error> {
        Ok(&self.resources)
    }

    async fn services<'a>(&'a self) -> Result<&'a Self::Services, Self::Error> {
        Ok(&self.services)
    }

    async fn members<'a>(&'a self) -> Result<&'a Self::Members, Self::Error> {
        Ok(&self.members)
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
