//! Cookies manager

use cookie_store::{Cookie, CookieStore as RawCookieStore, RawCookie, RawCookieParseError};
use reqwest::header::HeaderValue;

use std::fs::{File, OpenOptions};
use std::io::BufReader;
use std::ops::{Deref, DerefMut};
use std::path::Path;

use crate::cell::AtomicCell;

/// Cookie Store
pub struct CookieStore(RawCookieStore);

impl Default for CookieStore {
    fn default() -> Self {
        CookieStore(RawCookieStore::default())
    }
}

impl Deref for CookieStore {
    type Target = RawCookieStore;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for CookieStore {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl CookieStore {
    /// Load cookie store from file, if file not exist or invalid, return default store
    pub fn from_file<P: AsRef<Path>>(path: P) -> Self {
        let store = match File::open(&path) {
            Ok(f) => RawCookieStore::load_all(BufReader::new(f), |s| {
                serde_json::from_str::<Cookie<'_>>(s)
            })
            .unwrap(),
            Err(_) => RawCookieStore::default(),
        };
        CookieStore(store)
    }

    /// Save cookie store to file
    pub fn to_file<P: AsRef<Path>>(&self, path: P) {
        let mut file = match OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(path)
        {
            Ok(f) => f,
            Err(e) => {
                eprintln!("Failed to open cookie file: {e}");
                return;
            }
        };
        if let Err(e) = self.save_incl_expired_and_nonpersistent(&mut file, serde_json::to_string) {
            eprintln!("Failed to save cookie store: {e}");
        }
    }
}

/// Atomic Cookie store
pub(crate) struct AtomicCookieStore(AtomicCell<CookieStore>);

impl Default for AtomicCookieStore {
    fn default() -> Self {
        AtomicCookieStore::new(CookieStore::default())
    }
}

impl Deref for AtomicCookieStore {
    type Target = AtomicCell<CookieStore>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl AtomicCookieStore {
    /// Create a new atomic cookie store
    pub fn new(cookie_store: CookieStore) -> Self {
        AtomicCookieStore(AtomicCell::new(cookie_store))
    }
}

impl reqwest::cookie::CookieStore for AtomicCookieStore {
    fn set_cookies(
        &self,
        cookie_headers: &mut dyn Iterator<Item = &HeaderValue>,
        url: &reqwest::Url,
    ) {
        let cookies = cookie_headers.filter_map(|val| {
            std::str::from_utf8(val.as_bytes())
                .map_err(RawCookieParseError::from)
                .and_then(RawCookie::parse)
                .map(|c| c.into_owned())
                .ok()
        });
        self.update(|store| {
            store.store_response_cookies(cookies, url);
        });
    }

    fn cookies(&self, url: &reqwest::Url) -> Option<HeaderValue> {
        let s = self
            .load()
            .get_request_values(url)
            .map(|(name, value)| format!("{name}={value}"))
            .collect::<Vec<_>>()
            .join("; ");

        if s.is_empty() {
            return None;
        }

        HeaderValue::from_str(&s).ok()
    }
}
