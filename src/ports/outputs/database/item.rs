use std::hash::Hash;


pub trait Item: Sized + Clone + PartialEq {
    const NAME: &'static str;
    const FIELDS: &'static [&'static str];
    type PK: Clone + Hash + PartialEq;
    type SK: Clone + Hash + PartialEq;
}