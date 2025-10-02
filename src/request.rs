use reqwest::header::{HeaderMap, HeaderValue, USER_AGENT};

use std::sync::Arc;

pub use reqwest::Client;

#[inline]
pub fn client<C: reqwest::cookie::CookieStore + 'static>(cookies: Arc<C>) -> Client {
    const UA: &[u8] = b"Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/130.0.0.0 Safari/537.36 Edg/130.0.0.0";
    let mut header = HeaderMap::new();
    header.insert(USER_AGENT, HeaderValue::from_bytes(UA).expect("UA should always be valid"));

    Client::builder()
        .default_headers(header)
        .cookie_provider(cookies)
        .build()
        .expect("Client should always be built successfully")
}
