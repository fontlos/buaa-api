use cookie_store::{Cookie, CookieStore};
use reqwest::{Client, header::{HeaderMap, HeaderName, HeaderValue}};
use reqwest_cookie_store::CookieStoreMutex;
use thiserror::Error;

use std::fs::{self, File};
use std::io::BufReader;
use std::ops::Deref;
use std::path::PathBuf;
use std::sync::Arc;

/// This is the core of this crate, it is used to store cookies and send requests </br>
/// The prefix of most API names is derived from the fourth-level domain name of the corresponding domain name
#[derive(Debug)]
pub struct Session {
    client: Client,
    cookie_path: Option<PathBuf>,
    cookie_store: Arc<CookieStoreMutex>,
}

#[derive(Debug, Error)]
pub enum SessionError{
    #[error("Cookie Error")]
    CookieError,
    #[error("No Execution Value. BUT this is usually not an error, it means the cookie is still valid, causes redirects to a post-landing page, so you can't get the execution value")]
    NoExecutionValue,
    #[error("No token: {0}")]
    NoToken(String),
    #[error("Request Error")]
    RequestError(reqwest::Error),
    #[error("Login Error: {0}")]
    LoginError(String),
}

impl Session {
    /// Create a new session in memory, if you call `save` method, it will save cookies to `cookies.json` defaultly
    /// ```rust
    /// use buaa::Session;
    ///
    /// fn main() {
    ///     let session = Session::new_in_memory();
    ///     // if you call `save` method, it will save cookies to `cookies.json` defaultly
    ///     // session.save();
    /// }
    /// ```
    pub fn new_in_memory() -> Self {
        let cookie_store = Arc::new(CookieStoreMutex::new(CookieStore::default()));
        let mut header = HeaderMap::new();
        header.insert(HeaderName::from_bytes(b"User-Agent").unwrap(), HeaderValue::from_bytes(b"Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/130.0.0.0 Safari/537.36 Edg/130.0.0.0").unwrap());

        let client = Client::builder()
            .default_headers(header)
            .cookie_provider(cookie_store.clone())
            .build()
            .unwrap();

        Session {
            client,
            cookie_path: None,
            cookie_store,
        }
    }
    /// Create a new session in file, if the file is not exist, it will create a new one, but It won't be saved until you call `save` method
    /// ```rust
    /// use buaa::Session;
    ///
    /// fn main() {
    ///     let session = Session::new_in_file("path_to_cookies.json");
    ///     session.save();
    /// }
    /// ```
    pub fn new_in_file(path: &str) -> Self {
        let path = PathBuf::from(path);
        let cookie_store = match File::open(&path) {
            Ok(f) => CookieStore::load_all(
                BufReader::new(f),
                |s| serde_json::from_str::<Cookie>(s),
            ).unwrap(),
            Err(_) => CookieStore::default(),
        };

        let cookie_store = Arc::new(CookieStoreMutex::new(cookie_store));
        let mut header = HeaderMap::new();
        header.insert(HeaderName::from_bytes(b"User-Agent").unwrap(), HeaderValue::from_bytes(b"Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/130.0.0.0 Safari/537.36 Edg/130.0.0.0").unwrap());

        let client = Client::builder()
            .default_headers(header)
            .cookie_provider(cookie_store.clone())
            .build()
            .unwrap();

        Session {
            client,
            cookie_path: Some(path),
            cookie_store,
        }
    }
    /// save cookies manually
    pub fn save(&mut self) {
        let path = match &self.cookie_path {
            Some(p) => p.to_str().unwrap(),
            None => {
                "cookies.json"
            },
        };
        let mut file = match fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(path)
        {
            Ok(f) => f,
            Err(e) => {
                eprintln!("Failed to open cookie file: {:?}", e);
                return
            },
        };
        let store = self.cookie_store.lock().unwrap();
        if let Err(e) = store.save_incl_expired_and_nonpersistent(&mut file, |s| serde_json::to_string(s)) {
            eprintln!("Failed to save cookie store: {}", e);
        }
    }

    pub fn have_cookie_path(&self) -> bool {
        self.cookie_path.is_some()
    }
}

impl Deref for Session {
    type Target = Client;

    fn deref(&self) -> &Self::Target {
        &self.client
    }
}

impl From<reqwest::Error> for SessionError {
    fn from(e: reqwest::Error) -> Self {
        SessionError::RequestError(e)
    }
}
