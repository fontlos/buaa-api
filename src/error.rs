pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Server internal error. Usually, it is caused by an expired login
    #[error("Error From API: {0}")]
    APIError(String),
    /// The relevant Cookies or Token expires
    #[error("Login Expired: {0}")]
    LoginExpired(String),
    /// Some of the key information used for login is incorrect
    #[error("Login Error: {0}")]
    LoginError(String),

    #[error("Lock Error")]
    LockError,

    /// Crash in Serde
    #[error("Failed Deserialize: {0}")]
    DeserializeError(#[from] serde_json::Error),
    /// Crash in Reqwest
    #[error("Request Error")]
    RequestError(#[from] reqwest::Error),
}
