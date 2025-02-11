use std::hash::Hash;


pub trait Item: Sized + Clone + PartialEq {
    type PK: Clone + Hash + PartialEq;
    type SK: Clone + Hash + PartialEq;
}

impl<T: Item, U: Item> Item for (T, U) {
    type PK = (T::PK, U::PK);
    type SK = (T::SK, U::SK);
}