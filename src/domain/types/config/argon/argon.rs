use serde::{Serialize, Deserialize, Deserializer};
use serde::de::{self, Visitor, MapAccess};
use super::algorithm::Algorithm;
use super::version::Version;
use argon2::PasswordHasher;
use tokio::sync::OnceCell;
use super::params::Params;
use super::super::Secret;
use argon2::Argon2;
use std::fmt;



static PEPPER: OnceCell<String> = OnceCell::const_new();



#[derive(Clone, Debug, Default, Serialize)]
pub struct Argon {
    algorithm: Algorithm,
    version: Version,
    params: Params,
    pepper: Option<&'static String>,
    #[serde(skip)]
    argon2: Argon2<'static>
}

impl PartialEq for Argon {
    fn eq(&self, other: &Self) -> bool {
        self.algorithm == other.algorithm && self.version == other.version && self.params == other.params && self.pepper == other.pepper
    }
}


impl Argon {
    fn argon2(algorithm: Algorithm, version: Version, params: Params) -> Argon2<'static> {
        let algorithm = algorithm.into();
        let version = version.into();
        let params: argon2::Params = params.into();
        match PEPPER.get() {
            Some(secret) => Argon2::new_with_secret(secret.as_bytes(), algorithm, version, params.clone()).unwrap_or(Argon2::new(algorithm, version, params)),
            None => Argon2::new(algorithm, version, params)
        }
    }
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
                let mut params = Option::<Params>::None;
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
                            let new_pepper = <String as Secret>::process(&raw_pepper).map_err(de::Error::custom)?.unwrap_or(raw_pepper);
                            pepper = Some(new_pepper);
                        }
                        _ => {
                            let _: de::IgnoredAny = map.next_value()?;
                        }
                    }
                }

                let algorithm = algorithm.ok_or_else(|| de::Error::missing_field("algorithm"))?;
                let version = version.ok_or_else(|| de::Error::missing_field("version"))?;
                let params = params.ok_or_else(|| de::Error::missing_field("params"))?;
                match pepper {
                    Some(value) => PEPPER.set(value).map_err(de::Error::custom)?,
                    None => (),
                }
                let argon2 = Argon::argon2(algorithm, version, params.clone());
                let pepper = PEPPER.get();
                let argon = Argon{algorithm, version, params, pepper, argon2};
                Ok(argon)
            }
        }
        deserializer.deserialize_struct("Argon", &["algorithm", "version", "params", "pepper"], ArgonVisitor)
    }
}


impl PasswordHasher for Argon {
    type Params = <Argon2<'static> as PasswordHasher>::Params;
    
    fn hash_password_customized<'b>(&self, password: &[u8], algorithm: Option<argon2::password_hash::Ident<'b>>, version: Option<argon2::password_hash::Decimal>, params: Self::Params, salt: impl Into<argon2::password_hash::Salt<'b>>) -> argon2::password_hash::Result<argon2::PasswordHash<'b>> {
        self.argon2.hash_password_customized(password, algorithm, version, params, salt)
    }
}
