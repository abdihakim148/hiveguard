#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    NotFound,
    InvalidInput(String),
    DatabaseError(String),
    Unauthorized,
    Unknown(String),
}
