use bytes::Bytes;
use cookie_store::{CookieStore, RawCookie, RawCookieParseError, Cookie};
use reqwest::header::HeaderValue;

use std::fs::{File, OpenOptions};
use std::io::BufReader;
use std::path::Path;

use crate::cell::AtomicCell;

pub struct AtomicCookieStore(AtomicCell<CookieStore>);

impl Default for AtomicCookieStore {
    fn default() -> Self {
        AtomicCookieStore::new(CookieStore::default())
    }
}

impl AtomicCookieStore {
    pub fn new(cookie_store: CookieStore) -> AtomicCookieStore {
        AtomicCookieStore(AtomicCell::new(cookie_store))
    }

    pub fn load(&self) -> &CookieStore {
        self.0.load()
    }

    pub fn update<F>(&self, f: F)
    where
        F: FnOnce(&mut CookieStore),
    {
        self.0.update(f);
    }

    pub fn from_file<P: AsRef<Path>>(path: P) -> crate::Result<CookieStore> {
        let store = match File::open(&path) {
            Ok(f) => {
                CookieStore::load_all(BufReader::new(f), |s| serde_json::from_str::<Cookie<'_>>(s))
                    .unwrap()
            }
            Err(_) => CookieStore::default(),
        };

        Ok(store)
    }

    pub fn to_file<P: AsRef<Path>>(&self, path: P) {
        let mut file = match OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(path)
        {
            Ok(f) => f,
            Err(e) => {
                eprintln!("Failed to open cookie file: {:?}", e);
                return;
            }
        };
        let store = self.load();
        if let Err(e) =
            store.save_incl_expired_and_nonpersistent(&mut file, |s| serde_json::to_string(s))
        {
            eprintln!("Failed to save cookie store: {}", e);
        }
    }
}

impl reqwest::cookie::CookieStore for AtomicCookieStore {
    fn set_cookies(&self, cookie_headers: &mut dyn Iterator<Item = &HeaderValue>, url: &url::Url) {
        let cookies = cookie_headers.filter_map(|val| {
            std::str::from_utf8(val.as_bytes())
                .map_err(RawCookieParseError::from)
                .and_then(RawCookie::parse)
                .map(|c| c.into_owned())
                .ok()
        });
        self.0.update(|store| {
            store.store_response_cookies(cookies, url);
        });
    }

    fn cookies(&self, url: &url::Url) -> Option<HeaderValue> {
        let s = self
            .0
            .load()
            .get_request_values(url)
            .map(|(name, value)| format!("{}={}", name, value))
            .collect::<Vec<_>>()
            .join("; ");

        if s.is_empty() {
            return None;
        }

        HeaderValue::from_maybe_shared(Bytes::from(s)).ok()
    }
}
