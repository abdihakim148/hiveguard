/// Module for user-related types.
mod user;
/// Module for error-related types.
mod error;
mod number;
mod value;

/// Re-exporting error types for external access.
pub use error::*;
/// Re-exporting user types for external access.
pub use user::*;
pub use value::*;


pub type Result<T> = std::result::Result<T, Error>;