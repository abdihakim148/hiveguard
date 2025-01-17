use std::hash::Hash;


pub trait Item {
    type PK: Clone + Hash + PartialEq;
    type SK: Clone + Hash + PartialEq;
}