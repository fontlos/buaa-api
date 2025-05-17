pub(crate) use reqwest::Client;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};

use std::sync::Arc;

#[inline]
pub(crate) fn client<C: reqwest::cookie::CookieStore + 'static>(cookies: Arc<C>) -> Client {
    let mut header = HeaderMap::new();
    header.insert(HeaderName::from_bytes(b"User-Agent").unwrap(), HeaderValue::from_bytes(b"Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/130.0.0.0 Safari/537.36 Edg/130.0.0.0").unwrap());

    Client::builder()
        .default_headers(header)
        .cookie_provider(cookies)
        .build()
        .unwrap()
}

pub async fn retry<F, Fut, T, E>(
    ctx: &crate::Context<impl Send + Sync>,
    mut f: F,
    relogin: impl Fn() -> Fut,
) -> Result<T, E>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<T, E>>,
    E: std::fmt::Debug,
{
    let policy = ctx.policy.load();
    loop {
        let result = f().await;
        match &result {
            Ok(_) => return result,
            Err(_) if policy == &LoginPolicy::Auto => {
                // 先尝试 relogin
                relogin().await.ok();
                continue;
            }
            _ => return result,
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum LoginPolicy {
    Manual,
    Auto,
}

impl LoginPolicy {
    pub fn is_auto(&self) -> bool {
        matches!(self, LoginPolicy::Auto)
    }
}
