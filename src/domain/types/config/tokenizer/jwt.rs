use serde::{Serialize, Deserialize, de::DeserializeOwned};
use crate::domain::services::{Tokenizer, Result};

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct Jwt{}


impl<Payload: Serialize + DeserializeOwned + Send + Sync + 'static> Tokenizer<Payload> for Jwt {
    fn try_sign<Input: Into<Payload>>(&self, input: Input) -> Result<String> {
        todo!()
    }

    fn try_verify(&self, token_str: &str) -> Result<Payload> {
        todo!()
    }
}