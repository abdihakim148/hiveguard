/**
 * Trait for configuration management, providing methods to load and save configurations.
 *
 * This trait defines the necessary operations for handling configuration data,
 * including loading from and saving to a specified path.
 */
pub trait Config: Sized {
    type Error;
    /// This would be the default path for the configuration file.
    const PATH: &'static str = "./config.yaml";
    /// Loads the configuration from a specified file path.
    ///
    /// # Arguments
    ///
    /// * `path` - An optional string slice representing the path to load the configuration from.
    ///
    /// # Returns
    ///
    /// * `Result<Self, Self::Error>` - The loaded configuration wrapped in a `Result`.
    async fn load(path: Option<&str>) -> Result<Self, Self::Error>;
    /// Saves the configuration to a specified file path.
    ///
    /// # Arguments
    ///
    /// * `path` - An optional string slice representing the path to save the configuration to.
    ///
    /// # Returns
    ///
    /// * `Result<Self, Self::Error>` - The result of the save operation wrapped in a `Result`.
    async fn save(path: Option<&str>) -> Result<Self, Self::Error>;
}
