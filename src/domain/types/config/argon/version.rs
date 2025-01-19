use argon2::Version as ArgonVersion;
use serde::{Serialize, Deserialize, Serializer, Deserializer};
use serde::de::{self, Visitor};
use std::fmt;


#[derive(Copy, Clone, Default, PartialEq)]
#[repr(u8)]
pub enum Version {
    V0x10 = 16,
    #[default]
    V0x13 = 19
}


impl Serialize for Version {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u8(*self as u8)
    }
}

struct VersionVisitor;

impl<'de> Visitor<'de> for VersionVisitor {
    type Value = Version;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a u8 representing a Version")
    }

    fn visit_u8<E>(self, value: u8) -> Result<Version, E>
    where
        E: de::Error,
    {
        match value {
            16 => Ok(Version::V0x10),
            19 => Ok(Version::V0x13),
            _ => Err(de::Error::custom("unknown version")),
        }
    }
}

impl<'de> Deserialize<'de> for Version {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_u8(VersionVisitor)
    }
}


impl From<u8> for Version {
    fn from(value: u8) -> Self {
        match value {
            16 => Version::V0x10,
            19 => Version::V0x13,
            _ => Default::default()
        }
    }
}

impl From<Version> for u8 {
    fn from(version: Version) -> Self {
        match version {
            Version::V0x10 => 16,
            Version::V0x13 => 19
        }
    }
}


impl From<Version> for ArgonVersion {
    fn from(version: Version) -> Self {
        match version {
            Version::V0x10 => ArgonVersion::V0x10,
            Version::V0x13 => ArgonVersion::V0x13
        }
    }
}


impl From<ArgonVersion> for Version {
    fn from(value: ArgonVersion) -> Self {
        match value {
            ArgonVersion::V0x10 => Version::V0x10,
            ArgonVersion::V0x13 => Version::V0x13
        }
    }
}
