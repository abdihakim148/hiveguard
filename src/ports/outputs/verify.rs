use crate::ports::outputs::database::{Item, GetItem, CreateItem, DeleteItem, GetItems};
use serde::{de::DeserializeOwned, Serialize};
use crate::ports::ErrorTrait;
use std::rc::Rc;

/// A trait for verification services that support different contact types and verification methods.
///
/// This trait provides a flexible interface for initiating and verifying contact information
/// across different verification strategies and database backends.
///
/// # Type Parameters
/// * `T` - The type of contact being verified (e.g., email, phone number)
pub trait Verify<T: Clone, P: Clone = T>: DeserializeOwned + Sized {
    /// The type of verification code used for this verification process
    /// 
    /// Must implement both the `Code` trait and be storable as an `Item` 
    type Verification: Code<P, 6> + Item;

    /// The error type for verification operations
    type Error: ErrorTrait;

    /// The channel type for sending verification (e.g., SMS, email)
    type Channel;

    /// Initiates the verification process for a given contact
    ///
    /// # Arguments
    /// * `contact` - The contact to be verified
    /// * `channel` - The communication channel for verification
    /// * `db` - A database that can create verification codes
    ///
    /// # Returns
    /// A result indicating successful initiation or an error
    async fn initiate<DB: CreateItem<Self::Verification>>(
        &self,
        contact: &T, 
        channel: Self::Channel, 
        db: &DB
    ) -> Result<(), Self::Error>;

    /// Verifies a contact using a provided verification code
    ///
    /// # Arguments
    /// * `contact` - The contact being verified
    /// * `code` - The verification code to check
    /// * `db` - A database that can retrieve and delete verification codes
    ///
    /// # Returns
    /// A result indicating successful verification or an error
    async fn verify<DB: GetItem<Self::Verification>>(
        &self,
        contact: &T, 
        code: &str, 
        db: &DB
    ) -> Result<(), Self::Error>;
}

/// A trait representing a verification code
///
/// Provides methods for creating, accessing, and validating verification codes
pub trait Code<T: Clone, const DIGITS: usize = 6>: Sized {
    type Id: Serialize + DeserializeOwned;
    /// Creates a new verification code
    fn new(contact: &T, ttl: Option<i64>, id: Self::Id) -> Self;

    /// Retrieves the verification code as a reference-counted string
    fn code(&self) -> u32;

    /// Converts the code into a String for universal familiarity
    fn as_str(&self) -> String {
        let mut code = String::new();
        let string = self.code().to_string();
        if code.len() < DIGITS {
            let missing = DIGITS - code.len();
            for _ in 0..missing {
                code.push('0');
            }
        }
        code.push_str(&string);
        code
    }
}
