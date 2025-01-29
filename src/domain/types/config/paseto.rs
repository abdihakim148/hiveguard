use std::os::unix::fs::OpenOptionsExt; // For setting file permissions on Unix systems
use serde::{Serialize, Deserialize}; // For serializing and deserializing data
use std::io::{Read, Write, Error}; // For file I/O operations
use std::fs::{File, OpenOptions}; // For file handling
use super::super::PasetoKeys; // Importing PasetoKeys for key management


/// Default file path for storing Paseto keys
const DEFAULT_PATH: &'static str = "paseto_keys.json";


#[derive(Debug, Clone, PartialEq)] // Paseto struct with serialization capabilities
pub struct Paseto {
    /// File path for storing keys
    path: String,
    /// Paseto keys
    keys: PasetoKeys,
}


impl Paseto {
    /// Saves the Paseto keys to the specified file path.
    ///
    /// # Returns
    ///
    /// * `Result<(), Error>` - Result indicating success or failure of the save operation.
    fn save(&self) -> Result<(), Error> {
        let path = &self.path;
        let mut file = OpenOptions::new() // Open file with write permissions
            .write(true)
            .create(true)
            .truncate(true)
            .mode(0o600) // Set file permissions: owner can read and write
            .open(path)?;
        let keys = &self.keys; // Reference to Paseto keys
        let json = serde_json::to_string(keys)?; // Serialize keys to JSON
        let buf = json.as_bytes(); // Convert JSON to bytes
        file.write_all(buf)?;
        Ok(())
    }

    /// Loads the Paseto keys from the specified file path.
    ///
    /// # Arguments
    ///
    /// * `path` - A string slice representing the file path to load the keys from.
    ///
    /// # Returns
    ///
    /// * `Result<Self, Error>` - Result containing the loaded Paseto struct or an error.
    fn load(path: &str) -> Result<Self, Error> {
        let mut json = String::new(); // Buffer for file content
        File::open(path)?.read_to_string(&mut json)?;
        let path = path.to_string();
        let keys = serde_json::from_str::<PasetoKeys>(&json)?; // Deserialize JSON to PasetoKeys
        Ok(Self { path, keys }) // Return Paseto instance
    }
}


impl Serialize for Paseto {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer {
        serializer.serialize_str(&self.path)
    }
}

/// This function panics incase the data could not be written to the file
impl Default for Paseto {
    /// Provides a default instance of Paseto.
    ///
    /// This function attempts to load keys from the default path. If loading fails,
    /// it generates new keys, saves them, and returns the new instance.
    ///
    /// # Returns
    ///
    /// * `Self` - A default instance of Paseto.
    fn default() -> Self {
        let path = DEFAULT_PATH;
        match Self::load(path) {
            Ok(paseto) => paseto,
            _ => {
                let path = path.to_string();
                let keys = PasetoKeys::default();
                let paseto = Paseto { path, keys }; // Create new Paseto instance
                // Save the new keys, panicking if it fails
                paseto.save().unwrap();
                paseto
            }
        }
    }
}


impl<'de> Deserialize<'de> for Paseto {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let path = String::deserialize(deserializer)?;
        Ok(Paseto::load(&path).unwrap_or_default())
    }
}
