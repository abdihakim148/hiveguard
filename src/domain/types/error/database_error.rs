#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DatabaseError {
    ConnectionFailed(String),
}
