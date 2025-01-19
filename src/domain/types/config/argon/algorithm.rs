use argon2::Algorithm as ArgonAlgorithm;
use serde::{Serialize, Deserialize};


#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize, Default)]
pub enum Algorithm {
    Argon2d,
    Argon2i,
    #[default]
    Argon2id,
}


impl From<Algorithm> for ArgonAlgorithm {
    fn from(value: Algorithm) -> Self {
        match value {
            Algorithm::Argon2d => ArgonAlgorithm::Argon2d,
            Algorithm::Argon2i => ArgonAlgorithm::Argon2i,
            Algorithm::Argon2id => ArgonAlgorithm::Argon2id
        }
    }
}


impl From<ArgonAlgorithm> for Algorithm {
    fn from(value: ArgonAlgorithm) -> Self {
        match value {
            ArgonAlgorithm::Argon2d => Algorithm::Argon2d,
            ArgonAlgorithm::Argon2i => Algorithm::Argon2i,
            ArgonAlgorithm::Argon2id => Algorithm::Argon2id
        }
    }
}