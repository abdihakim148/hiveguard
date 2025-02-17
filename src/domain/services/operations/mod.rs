use crate::ports::outputs::database::{Item, GetItem};


mod user;

pub trait Get: Sized + Item {
    type Error;
    type Filter;

    async fn get<DB: GetItem<Self>>(filter: &Self::Filter, db: &DB) -> Result<Self, Self::Error>;
}