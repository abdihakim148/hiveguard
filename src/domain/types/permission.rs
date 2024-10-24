use serde::{Serialize, Deserialize};

/// Enum representing various permissions.
#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Permission {
    Create,
    Read,
    Update,
    Delete,
    Start,
    Pause,
    Stop,
    Cancel,
}
