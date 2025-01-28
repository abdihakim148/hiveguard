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



pub static PEPPER: OnceCell<String> = OnceCell::const_new();



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
    /// Creates a new Argon2 instance with the given algorithm, version, and parameters.
    ///
    /// # Arguments
    ///
    /// * `algorithm` - The algorithm to use for hashing.
    /// * `version` - The version of the Argon2 algorithm.
    /// * `params` - The parameters for the Argon2 hashing.
    ///
    /// # Returns
    ///
    /// An Argon2 instance configured with the provided settings.
    fn argon2(algorithm: Algorithm, version: Version, params: Params) -> Argon2<'static> {
        let algorithm = algorithm.into();
        let version = version.into();
        let params: argon2::Params = params.into();
        
        match PEPPER.get() {
            // If a secret is set, use it to create a new Argon2 instance with a secret
            Some(secret) => Argon2::new_with_secret(secret.as_bytes(), algorithm, version, params.clone()).unwrap_or(Argon2::new(algorithm, version, params)),
            None => Argon2::new(algorithm, version, params) // Create a standard Argon2 instance
        }
    }
}

impl<'de> Deserialize<'de> for Argon {
    /// Deserializes an Argon struct from a map of values.
    ///
    /// This implementation uses a custom visitor to handle the deserialization
    /// process, ensuring that all required fields are present and correctly
    /// initialized.
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
    
        struct ArgonVisitor; // Visitor for deserializing Argon

        impl<'de> Visitor<'de> for ArgonVisitor {
            type Value = Argon;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct Argon")
            }

            fn visit_map<V>(self, mut map: V) -> Result<Argon, V::Error> 
            where
                V: MapAccess<'de>,
            {
                let mut algorithm = None; // Placeholder for algorithm
                let mut version = None; // Placeholder for version
                let mut params = Option::<Params>::None; // Placeholder for params
                let mut pepper = Option::<String>::None; // Placeholder for pepper

                while let Some(key) = map.next_key()? {
                    match key {
                        "algorithm" => { // Deserialize algorithm
                            algorithm = Some(map.next_value()?);
                        }
                        "version" => { // Deserialize version
                            version = Some(map.next_value()?);
                        }
                        "params" => { // Deserialize params
                            params = Some(map.next_value()?);
                        }
                        "pepper" => { // Deserialize pepper
                            pepper = map.next_value()?;
                            if let Some(secret) = &pepper { // Process the pepper if present
                                let key = secret.as_str();
                                if let Some(value) = <String as Secret>::process(key).map_err(de::Error::custom)? {
                                    // Set the processed value back to pepper
                                    pepper = Some(value)
                                }
                            }
                        }
                        _ => {
                            let _: de::IgnoredAny = map.next_value()?; // Ignore unknown fields
                        }
                    }
                }

                let algorithm = algorithm.ok_or_else(|| de::Error::missing_field("algorithm"))?; // Ensure algorithm is present
                let version = version.ok_or_else(|| de::Error::missing_field("version"))?; // Ensure version is present
                let params = params.ok_or_else(|| de::Error::missing_field("params"))?; // Ensure params are present
                
                match pepper {
                    // Conditionally set the PEPPER based on the build type
                    Some(value) => { // If pepper is provided
                        #[cfg(test)] // In test builds, set without error handling
                        PEPPER.set(value);
                        #[cfg(not(test))] // In non-test builds, handle potential errors
                        PEPPER.set(value).map_err(de::Error::custom)?
                    },
                    None => (), // No action if pepper is not provided
                }
                let argon2 = Argon::argon2(algorithm, version, params.clone()); // Create Argon2 instance
                let pepper = PEPPER.get();
                let argon = Argon { algorithm, version, params, pepper, argon2 }; // Construct Argon
                Ok(argon)
            }
        }
        
        deserializer.deserialize_struct("Argon", &["algorithm", "version", "params", "pepper"], ArgonVisitor) // Deserialize Argon struct
    }
}


impl PasswordHasher for Argon {
    /// Hashes a password using the Argon2 algorithm with custom parameters.
    ///
    /// # Arguments
    ///
    /// * `password` - The password to hash.
    /// * `algorithm` - Optional algorithm identifier.
    /// * `version` - Optional version number.
    /// * `params` - Parameters for the hashing process.
    /// * `salt` - Salt value for the hashing process.
    ///
    /// # Returns
    ///
    /// A `PasswordHash` containing the hashed password.
    type Params = <Argon2<'static> as PasswordHasher>::Params;
    
    fn hash_password_customized<'b>(&self, password: &[u8], algorithm: Option<argon2::password_hash::Ident<'b>>, version: Option<argon2::password_hash::Decimal>, params: Self::Params, salt: impl Into<argon2::password_hash::Salt<'b>>) -> argon2::password_hash::Result<argon2::PasswordHash<'b>> {
        
        self.argon2.hash_password_customized(password, algorithm, version, params, salt) // Delegate to Argon2 instance
    }
}



#[cfg(test)]
mod tests {
    use super::*; // Import the necessary components for testing
    use serde_json::json;


    impl Argon {
        pub fn new() -> Self {
            let algorithm = Default::default(); // Use default algorithm
            let version = Default::default(); // Use default version
            let params: Params = Default::default(); // Use default params
            PEPPER.set(String::from("test_pepper")).unwrap(); // Set test pepper
            let pepper = PEPPER.get();
            let argon2 = Argon::argon2(algorithm, version, params.clone()); // Create Argon2 instance
            let algorithm = algorithm.into();
            let version = version.into();
            let params = params.into();
            let argon = Argon { algorithm, version, params, pepper, argon2 };
            argon
        }
    }

    #[test] // Test function for Argon deserialization
    fn test_argon_deserialization() {
        let argon = Argon::new(); // Construct Argon

        let json = serde_json::to_string(&argon).unwrap(); // Serialize Argon to JSON

        let de_argon: Argon = serde_json::from_str(&json).expect("Deserialization failed"); // Deserialize JSON back to Argon

        assert_eq!(de_argon, argon) // Assert that the deserialized Argon matches the original
    }
}
