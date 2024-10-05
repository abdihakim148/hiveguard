pub trait Table {
    type Item;

    fn create(&self, item: &Self::Item) -> Result<(), String>;
    fn read(&self, id: &str) -> Option<Self::Item>;
    fn update(&self, item: &Self::Item) -> Result<(), String>;
    fn delete(&self, id: &str) -> Result<(), String>;
}
