#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DatabaseError {
    ConnectionFailed(String),
    QueryFailed(String),
    TransactionFailed(String),
}
