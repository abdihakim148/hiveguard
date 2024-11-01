use super::{algorithm, Algorithm, Params, Version};
use serde::{Serialize, Deserialize};
use argon2::{Argon2, Error};

#[derive(Debug, Clone, Serialize, Copy, Deserialize, PartialEq, Default)]
#[serde(default)]
pub struct Argon {
    algorithm: Algorithm,
    version: Version,
    params: Params
}


impl<'key> TryFrom<Argon> for Argon2<'key> {
    type Error = Error;

    fn try_from(argon: Argon) -> Result<Self, Self::Error> {
        let (algorithm, version, params) = (argon.algorithm.into(), argon.version.into(), argon.params.try_into()?);
        Ok(Argon2::new(algorithm, version, params))
    }
}