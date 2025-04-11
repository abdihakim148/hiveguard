use std::hash::Hash;


pub trait Item: Sized + Clone + PartialEq {
    const NAME: &'static str;
    type PK: Clone + Hash + PartialEq;
    type SK: Clone + Hash + PartialEq;
}

impl<T: Item, U: Item> Item for (T, U) {
    const NAME: &'static str = U::NAME;
    type PK = (T::PK, U::PK);
    type SK = (T::SK, U::SK);
}