use argon2::Algorithm as ArgonAlgorithm;
use serde::{Serialize, Deserialize};


#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Default)]
pub enum Algorithm {
    Argon2d,
    Argon2i,
    #[default]
    Argon2id
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


impl PartialEq<ArgonAlgorithm> for Algorithm {
    fn eq(&self, other: &ArgonAlgorithm) -> bool {
        match self {
            Algorithm::Argon2d => {match other {ArgonAlgorithm::Argon2d => true, _ => false}},
            Algorithm::Argon2i => {match other {ArgonAlgorithm::Argon2i => true, _ => false}},
            Algorithm::Argon2id => {match other {ArgonAlgorithm::Argon2id => true, _ => false}}
        }
    }
}