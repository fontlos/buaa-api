pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Server internal error.
    #[error("API Error: {0}")]
    APIError(String),
    /// The relevant Cookies or Token expires
    #[error("Login Expired: {0}")]
    LoginExpired(Location),
    /// Some of the key information used for login is incorrect
    #[error("Login Error: {0}")]
    LoginError(String),

    /// Crash in Serde
    #[error("Deserialize Error: {0}")]
    DeserializeError(#[from] serde_json::Error),
    /// Crash in Reqwest
    #[error("Request Error")]
    RequestError(#[from] reqwest::Error),
    /// Other errors
    #[error(transparent)]
    Other(#[from] Box<dyn std::error::Error>),
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
