pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Server internal error. Usually, it is caused by an expired login
    #[error("Error From API: {0}")]
    APIError(String),
    /// The relevant Cookies or Token expires
    #[error("Login Expired: {0}")]
    LoginExpired(Location),
    /// Some of the key information used for login is incorrect
    #[error("Login Error: {0}")]
    LoginError(String),

    /// Crash in Serde
    #[error("Failed Deserialize: {0}")]
    DeserializeError(#[from] serde_json::Error),
    /// Crash in Reqwest
    #[error("Request Error")]
    RequestError(#[from] reqwest::Error),

    #[error(transparent)]
    Other(#[from] Box<dyn std::error::Error>),
}

#[derive(Debug, Eq, PartialEq)]
pub enum Location {
    SSO,
    ASS,
    APP,
    BOYA,
    CLASS,
    CLOUD,
    SPOC,
    SRS,
    TES,
    USER,
    WIFI,
}

impl std::fmt::Display for Location {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
