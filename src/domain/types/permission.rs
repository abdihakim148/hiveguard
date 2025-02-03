use serde::{Serialize, Deserialize};
use std::fmt::{Display, Formatter};
use std::error::Error as StdError;
use std::str::FromStr;

/// Enum representing various permissions.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Permission {
    Create,
    Read,
    Update,
    Delete,
}


impl Display for Permission {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        use Permission::*;
        match self {
            Create => write!(f, "create"),
            Read => write!(f, "read"),
            Update => write!(f, "update"),
            Delete => write!(f, "delete"),
        }
    }
}


impl FromStr for Permission {
    type Err = Box<dyn StdError>;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "create" => Ok(Self::Create),
            "read" => Ok(Self::Read),
            "update" => Ok(Self::Update),
            "delete" => Ok(Self::Delete),
            _ => Err("unknown permission")?
        }
    }
}