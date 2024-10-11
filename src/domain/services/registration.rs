/// A trait representing the registration process.
pub trait Registration {
    /// The type of the identifier for the registered entity.
    type Id;

    /// Registers a new entity.
    ///
    /// # Returns
    ///
    /// * `Result<Self::Id>` - Returns the ID of the registered entity wrapped in a `Result`.
    fn register(&self) -> Result<Self::Id, crate::domain::types::Error>;
}
