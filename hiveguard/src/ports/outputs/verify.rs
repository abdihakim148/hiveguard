use crate::ports::outputs::database::{Database, tables::VerificationsTable};
use rand::random_range;

pub trait Verify<Contact: Clone, const SIZE: usize = 6> {
    type VerificationCode: Code<Contact, SIZE, Error: Into<Self::Error>>;
    type Error;
    /// the transport channel for which the user should receive this code through. eg `SMS`, `Mail`, `Whatsapp`
    type Channel;

    async fn initiate<DB: Database<VerificationsTable: VerificationsTable<DB::Client, Item = Self::VerificationCode>>>(&self, contact: &Contact, channel: Self::Channel, magic_link_base_uri: Option<&str>, db: &DB) -> Result<Self::VerificationCode, Self::Error>;
    async fn verify<DB: Database<VerificationsTable: VerificationsTable<DB::Client, Item = Self::VerificationCode>>>(&self, contact: &Contact, code_or_id: &str, db: &DB) -> Result<(), Self::Error>;
}

pub trait Code<Contact, const SIZE: usize = 6> {
    type Error;
    const MIN: u32 = if SIZE == 1 { 0 } else { pow_10(SIZE - 1) };
    const MAX: u32 = pow_10(SIZE) - 1;
    fn new(contact: Contact, ttl: Option<i64>) -> Self;
    ///It is not recommended for you to manually implement this method.
    /// the default implementation is sufficient.
    /// if you have to manually implement this method. Make sure that the returned array is a valid string representation fo the expected digits or also manually implement the `as_str` method to make sure it is in it's correct representation.
    fn generate() -> [u8; SIZE] {
        let mut code = [0u8; SIZE];
        for digit in &mut code {
            *digit = random_range(48..=57);
        }
        code
    }
    fn code(&self) -> &[u8; SIZE];
    fn magic_link(base_uri: &str) -> String;
    /// The default implementation uses `unsafe` code which is actually safe if you stick with the default implementation of the `Self::generate` method.
    /// This implementation will always return a successful result as long as the `Self::generate` method does not change.
    fn as_str(&self) -> Result<&str, Self::Error> {
        Ok(unsafe{std::str::from_utf8_unchecked(self.code())})
    }
}


const fn pow_10(n: usize) -> u32 {
    let mut result = 1;
    let mut i = 0;
    while i < n {
        result *= 10;
        i += 1;
    }
    result
}