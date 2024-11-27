use thiserror::Error;

#[derive(Debug, Error)]
pub enum SessionError {
    /// Server internal error. Usually, it is caused by an expired login
    #[error("Error From API: {0}")]
    APIError(String),
    /// Crash in Serde
    #[error("Failed Deserialize: {0}")]
    DeserializeError(#[from] serde_json::Error),
    /// Some of the key information used for login is incorrect
    #[error("Login Error: {0}")]
    LoginError(String),
    /// The relevant Cookies or Token expires
    #[error("Login Expired: {0}")]
    LoginExpired(String),
    /// Crash in Reqwest
    #[error("Request Error")]
    RequestError(#[from] reqwest::Error),
}
