use crate::adaptors::outputs::database::memory::tables::organisations;
use crate::domain::types::{Either, Key, Value, Organisation, Error};
use crate::ports::outputs::database::{Table, Item};
use std::collections::HashMap;
use std::sync::RwLock;

#[derive(Default, Debug)]
pub struct Organisations {
    organisations: RwLock<HashMap<<Organisation as Item>::PK, Organisation>>,
    names: RwLock<HashMap<<Organisation as Item>::SK, <Organisation as Item>::PK>>
}


impl Table for Organisations {
    type Error = Error;
    type Item = Organisation;
    type Map = HashMap<String, Value>;
    const NAME: &'static str = "Organisations";
    
    async fn new() -> Result<Self, Self::Error> {
        Ok(Default::default())
    }

    async fn create(&self, item: &Self::Item) -> Result<<Self::Item as Item>::PK, Self::Error> {
        let (name, id) = (item.name.clone(), item.id);
        let (mut names, mut organisations) = (self.names.write()?, self.organisations.write()?);
        if let Some(_) = names.get(&name) {
            return Err(Error::Conflict(Self::Item::NAME))
        }
        if let Some(_) = organisations.get(&id) {
            return Err(Error::Conflict(Self::Item::NAME))
        }
        names.insert(name, id);
        organisations.insert(id, item.clone());
        Ok(id)
    }

    async fn get(&self, key: Either<&<Self::Item as Item>::PK, &<Self::Item as Item>::SK>) -> Result<Option<Self::Item>, Self::Error> {
        let pk = match key {
            Either::Left(pk) => *pk,
            Either::Right(sk) => {
                match self.names.read()?.get(sk) {
                    None => return Ok(None),
                    Some(pk) => *pk
                }
            }
        };
        Ok(self.organisations.read()?.get(&pk).cloned())
    }

    async fn get_many(&self, key: Key<&<Self::Item as Item>::PK, &<Self::Item as Item>::SK>) -> Result<Option<Vec<Self::Item>>, Self::Error> {
        unimplemented!()
    }

    async fn patch(&self, id: &<Self::Item as Item>::PK, mut map: Self::Map) -> Result<Self::Item, Self::Error> {
        let key = Either::Left(id);
        if let Some(organisation) = self.get(key).await? {
            let id = *id;
            let name = match map.remove("name") {
                Some(value) => value.try_into()?,
                None => organisation.name.clone()
            };
            let owners = match map.remove("owners") {
                Some(value) => value.try_into()?,
                None => organisation.owners.clone()
            };
            let domain = match map.remove("domain") {
                Some(value) => Some(value.try_into()?),
                None => organisation.domain.clone()
            };
            let home = match map.remove("home") {
                Some(value) => Some(value.try_into()?),
                None => organisation.home.clone()
            };
            let contacts = match map.remove("contacts") {
                Some(value) => value.try_into()?,
                None => organisation.contacts.clone()
            };
            let item = Organisation{id, name, owners, domain, home, contacts};
            self.update(&item).await?;
            return Ok(item);
        }
        Err(Error::NotFound(Self::Item::NAME))
    }

    async fn update(&self, item: &Self::Item) -> Result<(), Self::Error> {
        let mut organisations = self.organisations.write()?;
        let mut names = self.names.write()?;

        if let Some(id) = names.get(&item.name) {
            if id != &item.id {
                return Err(Error::Conflict(Self::Item::NAME));
            }
        }

        if let Some(existing_organisation) = organisations.get_mut(&item.id) {
            names.remove(&existing_organisation.name);
            *existing_organisation = item.clone();
            names.insert(item.name.clone(), item.id.clone());
        }else {
            self.create(item).await?;
        }

        Ok(())
    }

    async fn delete(&self, id: &<Self::Item as Item>::PK) -> Result<(), Self::Error> {
        let (mut organisations, mut names) = (self.organisations.write()?, self.names.write()?);
        let item = match organisations.remove(id) {
            Some(item) => item,
            None => return Ok(())
        };
        names.remove(&item.name);
        Ok(())
    }
}