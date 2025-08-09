use crate::api::Location;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Auth error.
    #[error("Auth Error: {0}")]
    Auth(AuthError),
    /// Network error
    #[error("Network Error: {0}")]
    Network(String),

    /// Crash in Serde
    #[error("Deserialize Error: {0}")]
    Deserialize(#[from] serde_json::Error),
    /// Crash in Reqwest
    #[error("Request Error")]
    Request(#[from] reqwest::Error),

    /// Server internal error. So you can't handle such errors, just for logging
    #[error("Server Error: {0}")]
    Server(String),

    /// Other errors
    #[error(transparent)]
    Other(#[from] Box<dyn std::error::Error>),
}

impl Error {
    pub fn auth_expired(location: Location) -> Self {
        Error::Auth(AuthError::Expired(location))
    }
}

#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error("No Username")]
    NoUsername,
    #[error("No Password")]
    NoPassword,
    /// Relevant Cookies or Token expires
    #[error("Auth Expired at: {0}")]
    Expired(Location),
}
