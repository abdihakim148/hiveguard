mod algorithm;
mod version;
mod params;


use serde::{Serialize, Deserialize};
use algorithm::Algorithm;
use version::Version;
use params::Params;
use argon2::Argon2;



#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct Argon {
    algorithm: Algorithm,
    version: Version,
    params: Params,
    pepper: Option<String>
}



impl<'a> From<&'a Argon> for Argon2<'a> {
    fn from(value: &'a Argon) -> Self {
        let algorithm = value.algorithm.into();
        let version = value.version.into();
        let params = value.params.clone().into();
        if let Some(pepper) = &value.pepper {
            let secret = pepper.as_bytes();
            return  Argon2::new_with_secret(secret, algorithm, version, params).unwrap_or_default();
        }
        Argon2::new(algorithm, version, params)
    }
}