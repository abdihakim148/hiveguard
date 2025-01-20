use serde::{Serialize, Deserialize, Deserializer};
use serde::de::{self, Visitor, MapAccess};
use super::algorithm::Algorithm;
use super::version::Version;
use super::params::Params;
use super::super::Secret;
use argon2::Argon2;
use std::fmt;



#[derive(Clone, Debug, PartialEq, Default, Serialize)]
pub struct Argon {
    algorithm: Algorithm,
    version: Version,
    params: Params,
    pepper: Option<String>
}

impl<'de> Deserialize<'de> for Argon {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct ArgonVisitor;

        impl<'de> Visitor<'de> for ArgonVisitor {
            type Value = Argon;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct Argon")
            }

            fn visit_map<V>(self, mut map: V) -> Result<Argon, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut algorithm = None;
                let mut version = None;
                let mut params = None;
                let mut pepper = None;

                while let Some(key) = map.next_key()? {
                    match key {
                        "algorithm" => {
                            algorithm = Some(map.next_value()?);
                        }
                        "version" => {
                            version = Some(map.next_value()?);
                        }
                        "params" => {
                            params = Some(map.next_value()?);
                        }
                        "pepper" => {
                            let raw_pepper: String = map.next_value()?;
                            pepper = <String as Secret>::process(&raw_pepper).map_err(de::Error::custom)?;
                        }
                        _ => {
                            let _: de::IgnoredAny = map.next_value()?;
                        }
                    }
                }

                let algorithm = algorithm.ok_or_else(|| de::Error::missing_field("algorithm"))?;
                let version = version.ok_or_else(|| de::Error::missing_field("version"))?;
                let params = params.ok_or_else(|| de::Error::missing_field("params"))?;

                Ok(Argon {
                    algorithm,
                    version,
                    params,
                    pepper,
                })
            }
        }

        deserializer.deserialize_struct("Argon", &["algorithm", "version", "params", "pepper"], ArgonVisitor)
    }
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
