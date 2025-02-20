use crate::ports::outputs::database::{Item, GetItem, UpdateItem, Map};
use crate::domain::types::Value;
use std::collections::HashMap;


mod user;

pub trait Get: Sized + Item {
    type Error;
    type Filter;

    async fn get<DB: GetItem<Self>>(filter: &Self::Filter, db: &DB) -> Result<Self, Self::Error>;
}


pub trait Update: Sized + Item {
    type Error;
    type Filter;

    async fn update<DB: UpdateItem<Self, Update = Map> + GetItem<Self>>(filter: &Self::Filter, db: &DB, item: HashMap<String, Value>) -> Result<Self, Self::Error>;
}

