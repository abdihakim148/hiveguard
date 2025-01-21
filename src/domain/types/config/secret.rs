use std::error::Error as StdError;
use std::env::var;


/// A trait for handling secrets with varying levels of security.
/// 
/// This trait provides a mechanism to retrieve secrets from different sources
/// based on the level of security required. It defines three levels of security:
/// single, double, and triple, each corresponding to a different method of
/// retrieving the secret.
///
/// # Associated Types
///
/// * `Error` - The error type that can be returned when processing secrets.
///
/// # Constants
///
/// * `DELIMETER` - A byte used to determine the level of security for the secret.
///
/// # Methods
///
/// * `process` - Determines the level of security and retrieves the secret accordingly.
/// * `single` - Retrieves a secret from an environment variable.
/// * `double` - Retrieves a secret from a more secure location than environment variables.
/// * `tripple` - Retrieves a secret from a maximum security location.
pub trait Secret: Sized {
    type Error;
    const DELIMETER: u8 = b'$';
    /// Determines the level of security and retrieves the secret accordingly.
    ///
    /// This method inspects the prefix of the key to determine the security level
    /// and calls the appropriate method (`single`, `double`, or `tripple`) to
    /// retrieve the secret.
    ///
    /// # Arguments
    ///
    /// * `key` - A string slice that holds the key for which the secret is to be retrieved.
    ///
    /// # Returns
    ///
    /// * `Result<Option<Self>, <Self as Secret>::Error>` - Returns an optional secret if successful, or an error if not.
    fn process(key: &str) -> Result<Option<Self>, <Self as Secret>::Error> {
        // Extract the first three bytes of the key to determine the security level.
        let mut bytes = match key.as_bytes().get(0..3) {
            Some(elements) => elements,
            None => key.as_bytes()
        }.into_iter().peekable();
        
        // Initialize a counter to track the number of delimiters found.
        let mut round = 0u8;
        // Count the number of delimiters to determine the security level.
        while let Some(byte) = bytes.peek() { 
            if *byte == &Self::DELIMETER {
                bytes.next();
                round += 1;
            }
        }
        
        // Call the appropriate method based on the number of delimiters found.
        match round {
            0 => Ok(None),
            1 => Ok(Some(Self::single(key)?)),
            2 => Ok(Some(Self::double(key)?)),
            3 => Ok(Some(Self::tripple(key)?)),
            _ => Ok(None),
        }
    }
    /// Retrieves a secret from an environment variable.
    ///
    /// # Arguments
    ///
    /// * `key` - A string slice that holds the name of the environment variable.
    ///
    /// # Returns
    ///
    /// * `Result<Self, <Self as Secret>::Error>` - Returns the secret if successful, or an error if not.
    fn single(key: &str) -> Result<Self, <Self as Secret>::Error>;
    /// Retrieves a secret from a more secure location than environment variables.
    ///
    /// # Arguments
    ///
    /// * `key` - A string slice that holds the key for the secure location.
    ///
    /// # Returns
    ///
    /// * `Result<Self, <Self as Secret>::Error>` - Returns the secret if successful, or an error if not.
    fn double(key: &str) -> Result<Self, <Self as Secret>::Error>;
    /// Retrieves a secret from a maximum security location.
    ///
    /// # Arguments
    ///
    /// * `key` - A string slice that holds the key for the maximum security location.
    ///
    /// # Returns
    ///
    /// * `Result<Self, <Self as Secret>::Error>` - Returns the secret if successful, or an error if not.
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
