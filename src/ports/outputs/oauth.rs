use crate::ports::ErrorTrait;
use oauth2::TokenResponse;
use url::Url;

/// Trait defining OAuth authentication flow
pub trait OAuth {
    // Using the associated type from the original trait
    type Error: ErrorTrait;

    /// Generate authorization URL for initiating OAuth flow
    async fn authorization_url(&self, provider: &str, redirect_url: &Url) -> Result<Url, Self::Error>;

    /// Authenticate user using provider and authorization code
    // Keeping 'impl TokenResponse' as it might work if the trait is not used as a dyn Trait object extensively.
    // If compilation fails later, this might need adjustment (e.g., to Box<dyn TokenResponse>).
    async fn authenticate(&self, provider: &str, code: &str) -> Result<impl TokenResponse, Self::Error>;
}