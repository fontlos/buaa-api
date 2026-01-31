//! Error handling

use std::borrow::Cow;
use std::error::Error as StdError;
use std::fmt::Display;

/// A Result alias
pub type Result<T> = std::result::Result<T, Error>;

type BoxError = Box<dyn StdError + Send + Sync>;

/// The Error type
#[derive(Debug)]
pub struct Error {
    kind: Kind,
    message: Cow<'static, str>,
    label: Option<&'static str>,
    code: Option<Code>,
    source: Option<BoxError>,
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        self.source.as_ref().map(|e| &**e as _)
    }
}

impl Error {
    pub(crate) fn new(kind: Kind, message: impl Into<Cow<'static, str>>) -> Error {
        Error {
            kind,
            message: message.into(),
            label: None,
            code: None,
            source: None,
        }
    }

    pub(crate) fn with_label(mut self, label: &'static str) -> Self {
        self.label = Some(label);
        self
    }

    pub(crate) fn with_code(mut self, code: Code) -> Self {
        self.code = Some(code);
        self
    }

    pub(crate) fn with_source<E>(mut self, source: E) -> Self
    where
        E: Into<BoxError>,
    {
        self.source = Some(source.into());
        self
    }

    pub(crate) fn auth(message: impl Into<Cow<'static, str>>) -> Self {
        Self::new(Kind::Auth, message)
    }

    pub(crate) fn io(message: impl Into<Cow<'static, str>>) -> Self {
        Self::new(Kind::Io, message)
    }

    pub(crate) fn network(message: impl Into<Cow<'static, str>>) -> Self {
        Self::new(Kind::Network, message)
    }

    pub(crate) fn parameter(message: impl Into<Cow<'static, str>>) -> Self {
        Self::new(Kind::Parameter, message)
    }

    pub(crate) fn parse(message: impl Into<Cow<'static, str>>) -> Self {
        Self::new(Kind::Parse, message)
    }

    pub(crate) fn server(message: impl Into<Cow<'static, str>>) -> Self {
        Self::new(Kind::Server, message)
    }

    /// Get the kind of error
    pub fn kind(&self) -> &Kind {
        &self.kind
    }

    /// Get the code of error
    pub fn code(&self) -> Option<&Code> {
        self.code.as_ref()
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}]", self.kind)?;
        if let Some(label) = self.label {
            write!(f, "::[{}]", label)?;
        }
        write!(f, ": {}", self.message)?;
        if let Some(code) = &self.code {
            write!(f, " (code: {:?})", code)?;
        }
        if let Some(source) = &self.source {
            write!(f, "\n    Caused by: {}", source)?;
            let mut cur = source.source();
            while let Some(c) = cur {
                write!(f, "\n        caused by: {}", c)?;
                cur = c.source();
            }
        }
        Ok(())
    }
}

/// The kind of error
#[derive(Debug)]
pub enum Kind {
    /// Authentication error
    Auth,
    /// I/O error
    Io,
    /// Network error
    Network,
    /// Parameter error
    Parameter,
    /// Parse error
    Parse,
    /// Server internal error. So you can't handle such errors, just for logging
    Server,
}

impl Display for Kind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Kind::Auth => write!(f, "AuthorizationError"),
            Kind::Io => write!(f, "IOError"),
            Kind::Network => write!(f, "NetworkError"),
            Kind::Parameter => write!(f, "ParameterError"),
            Kind::Parse => write!(f, "ParseError"),
            Kind::Server => write!(f, "ServerInternalError"),
        }
    }
}

/// The specific error code
#[derive(Debug)]
pub enum Code {
    // Authentication errors
    /// No Username
    AuthNoUsername,
    /// No Password
    AuthNoPassword,
    // 在自动刷新机制的帮助下, 这通常不会发生
    /// No Token
    AuthNoToken,

    // Network errors
    /// No local IP address
    NetworkNoLocalIp,
    // 只要网络连接正常就能断定这是因为没有连接到 BUAA-WiFi
    /// DNS resolution failure. It almost cause by not connect to BUAA-WiFi.
    NetworkDnsFailure,
}

impl From<reqwest::Error> for Error {
    fn from(value: reqwest::Error) -> Self {
        Error::network("From reqwest crate").with_source(value)
    }
}

impl From<serde_json::Error> for Error {
    fn from(value: serde_json::Error) -> Self {
        Error::parse("From serde_json crate").with_source(value)
    }
}
