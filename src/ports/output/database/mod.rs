pub mod table;

pub trait Database {
    async fn new<T>(args: T) -> Self;
}
