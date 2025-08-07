pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Auth error.
    #[error("Auth Error: {0}")]
    AuthError(AuthError),
    /// Network error
    #[error("Network Error: {0}")]
    NetworkError(String),

    /// Crash in Serde
    #[error("Deserialize Error: {0}")]
    DeserializeError(#[from] serde_json::Error),
    /// Crash in Reqwest
    #[error("Request Error")]
    RequestError(#[from] reqwest::Error),

    /// Server internal error. So you can't handle such errors, just for logging
    #[error("Server Error: {0}")]
    ServerError(String),

    /// Other errors
    #[error(transparent)]
    Other(#[from] Box<dyn std::error::Error>),
}

impl Error {
    pub fn auth_expired(location: Location) -> Self {
        Error::AuthError(AuthError::Expired(location))
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum Location {
    Ass,
    App,
    Boya,
    Class,
    Cloud,
    Spoc,
    Srs,
    Sso,
    Tes,
    User,
    Wifi,
}

impl std::fmt::Display for Location {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
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
