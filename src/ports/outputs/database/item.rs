use std::hash::Hash;


pub trait Item: Sized + Clone + PartialEq {
    type PK: Clone + Hash + PartialEq;
    type SK: Clone + Hash + PartialEq;
}