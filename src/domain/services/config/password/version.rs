use argon2::Version as ArgonVersion;
use serde::{Serialize, Deserialize};


#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Default)]
pub enum Version {
    V16,
    #[default]
    V19
}


impl From<Version> for ArgonVersion {
    fn from(value: Version) -> Self {
        match value {
            Version::V16 => ArgonVersion::V0x10,
            Version::V19 => ArgonVersion::V0x13
        }
    }
}

impl PartialEq<ArgonVersion> for Version {
    fn eq(&self, other: &ArgonVersion) -> bool {
        match self {
            Version::V16 => {match other {ArgonVersion::V0x10 => true, _ => false}},
            Version::V19 => {match other {ArgonVersion::V0x13 => true, _ => false}}
        }
    }
}