use thiserror::Error;

#[derive(Debug, Error)]
pub enum SessionError {
    #[error("Failed Deserialize: {0}")]
    DeserializeError(#[from] serde_json::Error),
    #[error("Login Error: {0}")]
    LoginError(String),
    #[error("Login Expired: {0}")]
    LoginExpired(String),
    #[error("Request Error")]
    RequestError(#[from] reqwest::Error),
}
