//! Error handling for `buaa-api`

use std::borrow::Cow;

use crate::api::Location;

/// `Result` type for `buaa-api`
pub type Result<T> = std::result::Result<T, Error>;

/// Error type for `buaa-api`
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Auth error.
    #[error("Auth Error: {0}")]
    Auth(#[from] AuthError),
    /// Network error
    #[error("Network Error: {0}")]
    Network(#[from] NetworkError),
    /// Parse error. Usually you cannot handle such errors, just for logging
    #[error("Parse Error: {0}")]
    Parse(#[from] ParseError),

    /// Server internal error. So you can't handle such errors, just for logging
    #[error("Server Error: {0}")]
    Server(Cow<'static, str>),

    /// Other errors
    #[error(transparent)]
    Other(#[from] Box<dyn std::error::Error>),
}

impl Error {
    #[inline]
    pub(crate) fn server(msg: impl Into<Cow<'static, str>>) -> Self {
        Error::Server(msg.into())
    }
}

/// Auth error
#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    /// No Username
    #[error("No Username")]
    NoUsername,
    /// No Password
    #[error("No Password")]
    NoPassword,
    // 在自动刷新机制的帮助下, 这通常不会发生
    /// No Token
    #[error("No Token: {0}")]
    NoToken(Location),
}

/// Network error
#[derive(Debug, thiserror::Error)]
pub enum NetworkError {
    /// Reqwest error
    #[error("Reqwest Error: {0}")]
    Reqwest(reqwest::Error),
    /// Cannot get local IP address
    #[error("Cannot get local IP address")]
    NoIp,
    /// Not connect to BUAA-WiFi
    #[error("Not connect to BUAA-WiFi")]
    NoBuaaWifi,
}

impl From<reqwest::Error> for Error {
    fn from(value: reqwest::Error) -> Self {
        Error::Network(NetworkError::Reqwest(value))
    }
}

/// Parse error
#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    /// Serde error. This usually caused by server.
    #[error("Serde Error: {0}")]
    Serde(serde_json::Error),
}

impl From<serde_json::Error> for Error {
    fn from(value: serde_json::Error) -> Self {
        Error::Parse(ParseError::Serde(value))
    }
}
