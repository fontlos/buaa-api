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
