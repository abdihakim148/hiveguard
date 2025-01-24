use crate::ports::Result;


pub trait Mailer: Sized {
    type Config;
    type Mail;

    /// Creates a new instance of the mailer with the given configuration.
    ///
    /// # Arguments
    ///
    /// * `config` - Configuration settings for the mailer.
    ///
    /// # Returns
    ///
    /// * `Result<Self>` - Returns a new instance of the mailer wrapped in a `Result`.
    async fn new(config: Self::Config) -> Result<Self>;

    /// Sends an email.
    ///
    /// # Arguments
    ///
    /// * `mail` - The mail details to be sent.
    ///
    /// # Returns
    ///
    /// * `Result<()>` - Returns `Ok(())` if the email is sent successfully, or an error otherwise.
    async fn send(&self, mail: Self::Mail) -> Result<()>;
}
