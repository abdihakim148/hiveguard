mod params;
mod version;
mod algorithm;
mod argon;


pub use params::*;
pub use version::*;
pub use algorithm::*;
pub use argon::*;


use static_init::dynamic;
use argon2::Argon2;


#[dynamic]
static ARGON2: Argon2<'static> = Argon::default().try_into().unwrap();