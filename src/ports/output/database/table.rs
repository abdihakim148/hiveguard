pub trait Table {
    type Item;
    type Id;

    fn create(&self, item: &Self::Item) -> Result<Self::Id, String>;
    fn read(&self, id: &Self::Id) -> Option<Self::Item>;
    fn update(&self, item: &Self::Item) -> Result<Self::Id, String>;
    fn delete(&self, id: &Self::Id) -> Result<Self::Id, String>;
}
