use crate::domain::types::{Either, Key, Value, Organisation, Error};
use crate::ports::outputs::database::{Table, Item};
use std::collections::HashMap;


pub struct Organisations;


impl Table for Organisations {
    type Error = Error;
    type Item = Organisation;
    type Map = HashMap<String, Value>;
    const NAME: &'static str = "Organisations";
    
    async fn new() -> Result<Self, Self::Error> {
        todo!()
    }

    async fn create(&self, item: &Self::Item) -> Result<<Self::Item as Item>::PK, Self::Error> {
        todo!()
    }

    async fn get(&self, key: Either<&<Self::Item as Item>::PK, &<Self::Item as Item>::SK>) -> Result<Option<Self::Item>, Self::Error> {
        todo!()
    }

    async fn get_many(&self, key: Key<&<Self::Item as Item>::PK, &<Self::Item as Item>::SK>) -> Result<Option<Vec<Self::Item>>, Self::Error> {
        todo!()
    }

    async fn patch(&self, id: &<Self::Item as Item>::PK, map: Self::Map) -> Result<Self::Item, Self::Error> {
        todo!()
    }

    async fn update(&self, item: &Self::Item) -> Result<(), Self::Error> {
        todo!()
    }

    async fn delete(&self, id: &<Self::Item as Item>::PK) -> Result<(), Self::Error> {
        todo!()
    }
}