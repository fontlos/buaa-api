pub(crate) use reqwest::Client;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};

use std::sync::Arc;

#[inline]
pub(crate) fn client<C: reqwest::cookie::CookieStore + 'static>(cookies: Arc<C>) -> Client {
    const UA: &[u8] = b"Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/130.0.0.0 Safari/537.36 Edg/130.0.0.0";
    let mut header = HeaderMap::new();
    header.insert(
        HeaderName::from_bytes(b"User-Agent").unwrap(),
        HeaderValue::from_bytes(UA).unwrap(),
    );

    Client::builder()
        .default_headers(header)
        .cookie_provider(cookies)
        .build()
        .unwrap()
}
