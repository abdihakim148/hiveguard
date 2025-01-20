mod argon;

use std::error::Error as StdError;
use std::env::var;
// use super::*;

#[allow(unused)]
pub trait Secret: Sized {
    type Error;
    const DELIMETER: u8 = b'$';
    fn process(key: &str) -> Result<Option<Self>, <Self as Secret>::Error> {
        let mut bytes = match key.as_bytes().get(0..3){
            Some(elements) => elements,
            None => key.as_bytes()
        }.into_iter().peekable();
        let mut round = 0u8;
        while let Some(byte) = bytes.peek() {
            if *byte == &Self::DELIMETER {
                bytes.next();
                round+=1;
            }
        }
        match round {
            0 => Ok(None),
            1 => Ok(Some(Self::single(key)?)),
            2 => Ok(Some(Self::double(key)?)),
            3 => Ok(Some(Self::tripple(key)?)),
            _ => Ok(None)
        }
    }
    /// This function is used to retrieve a secret from an env variable.
    fn single(key: &str) -> Result<Self, <Self as Secret>::Error>;
    /// This function will be used to retrieve a secret from a place more secure than environment variables.
    fn double(key: &str) -> Result<Self, <Self as Secret>::Error>;
    /// This function will be used to retrieve a secret from a place a maximum security.
    fn tripple(key: &str) -> Result<Self, <Self as Secret>::Error>;
}


impl<T> Secret for T 
where
     T: TryFrom<String>,
     T::Error: StdError + 'static,
{
    type Error = Box<dyn StdError + 'static>;
    fn single(key: &str) -> Result<Self, <Self as Secret>::Error> {
        Ok(var(key)?.try_into()?)
    }

    fn double(_key: &str) -> Result<Self, <Self as Secret>::Error> {
        Err("")?
    }

    fn tripple(_key: &str) -> Result<Self, <Self as Secret>::Error> {
        Err("")?
    }
}