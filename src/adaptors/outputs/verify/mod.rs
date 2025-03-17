#[cfg(any(feature = "twilio-email", feature = "twilio-phone"))]
mod twilio;
#[cfg(all(feature = "smtp", feature = "email"))]
mod smtp;
mod error;


pub use error::*;


#[cfg(any(feature = "twilio-email", feature = "twilio-phone"))]
pub type Verifyer = twilio::Twilio;
#[cfg(feature = "smtp")]
pub type Verifyer = smtp::Smtp;