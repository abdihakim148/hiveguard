use serde::{Serialize, Deserialize, de::DeserializeOwned};
use crate::domain::services;


pub mod paseto;
pub mod jwt;

use paseto::*;
use jwt::*;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Tokenizer {
    Jwt(Jwt),
    Paseto(Paseto)
}


impl Default for Tokenizer {
    fn default() -> Self {
        Self::Paseto(Default::default())
    }
}


impl<Payload: Serialize + DeserializeOwned + Send + Sync + 'static> services::Tokenizer<Payload> for Tokenizer {
    fn try_sign<Input: Into<Payload>>(&self, input: Input) -> services::Result<String> {
        match &self {
            Self::Jwt(jwt) => jwt.try_sign(input),
            Self::Paseto(paseto) => paseto.try_sign(input)
        }
    }

    fn try_verify(&self, token_str: &str) -> services::Result<Payload> {
        match &self {
            Self::Jwt(jwt) => jwt.try_verify(token_str),
            Self::Paseto(paseto) => paseto.try_verify(token_str)
        }
    }
}