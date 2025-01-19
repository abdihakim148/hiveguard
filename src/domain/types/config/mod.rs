mod password;
mod argon;

use std::error::Error as StdError;
use std::env::var;
use super::*;

pub use password::*;

#[allow(unused)]
pub trait Secret: Sized {
    type Error;
    const CHAR: char = '$';
    /// This function is used to retrieve a secret from an env variable.
    fn single(key: &str) -> Result<Self, Self::Error>;
    /// This function will be used to retrieve a secret from a place more secure than environment variables.
    fn double(key: &str) -> Result<Self, Self::Error>;
    /// This function will be used to retrieve a secret from a place a maximum security.
    fn tripple(key: &str) -> Result<Self, Self::Error>;
}


impl<T> Secret for T 
where
     T: TryFrom<String>,
     T::Error: StdError + 'static,
{
    type Error = Box<dyn StdError + 'static>;
    fn single(key: &str) -> Result<Self, Self::Error> {
        Ok(var(key)?.try_into()?)
    }

    fn double(_key: &str) -> Result<Self, Self::Error> {
        Err("")?
    }

    fn tripple(_key: &str) -> Result<Self, Self::Error> {
        Err("")?
    }
}