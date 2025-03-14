#[cfg(any(feature = "twilio-email", feature = "twilio-phone"))]
mod twilio;
#[cfg(all(feature = "smtp", feature = "email"))]
mod smtp;
mod error;

pub type Verifyer = twilio::Twilio;
pub use error::*;