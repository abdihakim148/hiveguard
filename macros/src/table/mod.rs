mod table;
mod method;
mod path;

pub use table::*;
pub use method::*;
pub use path::*;


type Result<T> = std::result::Result<T, syn::Error>;