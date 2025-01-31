use std::hash::Hash;


pub trait Item: Sized + Clone + PartialEq {
    const NAME: &'static str = "";
    type PK: Clone + Hash + PartialEq;
    type SK: Clone + Hash + PartialEq;
}