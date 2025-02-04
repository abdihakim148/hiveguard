use crate::adaptors::outputs::database::memory::tables::organisations;
use crate::domain::types::{Either, Key, Value, Organisation, Error};
use crate::ports::outputs::database::{Table, Item};
use std::collections::HashMap;
use std::sync::RwLock;

#[derive(Default, Debug)]
pub struct Organisations {
    primary: RwLock<HashMap<<Organisation as Item>::PK, Organisation>>,
    secondary: RwLock<HashMap<<Organisation as Item>::SK, <Organisation as Item>::PK>>
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
        let (mut secondary, mut primary) = (self.secondary.write()?, self.primary.write()?);
        if let Some(_) = secondary.get(&name) {
            return Err(Error::Conflict(Self::Item::NAME))
        }
        if let Some(_) = primary.get(&id) {
            return Err(Error::Conflict(Self::Item::NAME))
        }
        secondary.insert(name, id);
        primary.insert(id, item.clone());
        Ok(id)
    }

    async fn get(&self, key: Either<&<Self::Item as Item>::PK, &<Self::Item as Item>::SK>) -> Result<Option<Self::Item>, Self::Error> {
        let pk = match key {
            Either::Left(pk) => *pk,
            Either::Right(sk) => {
                match self.secondary.read()?.get(sk) {
                    None => return Ok(None),
                    Some(pk) => *pk
                }
            }
        };
        Ok(self.primary.read()?.get(&pk).cloned())
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
        let mut primary = self.primary.write()?;
        let mut secondary = self.secondary.write()?;

        if let Some(id) = secondary.get(&item.name) {
            if id != &item.id {
                return Err(Error::Conflict(Self::Item::NAME));
            }
        }

        if let Some(existing_organisation) = primary.get_mut(&item.id) {
            secondary.remove(&existing_organisation.name);
            *existing_organisation = item.clone();
            secondary.insert(item.name.clone(), item.id.clone());
        }else {
            self.create(item).await?;
        }

        Ok(())
    }

    async fn delete(&self, id: &<Self::Item as Item>::PK) -> Result<(), Self::Error> {
        let (mut primary, mut secondary) = (self.primary.write()?, self.secondary.write()?);
        let item = match primary.remove(id) {
            Some(item) => item,
            None => return Ok(())
        };
        secondary.remove(&item.name);
        Ok(())
    }
}